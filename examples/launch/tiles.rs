//! Background tile loader: a worker thread turns tile requests into RGBA
//! pixels (region images from `assets/map`, or the synthetic terrain and
//! geodata of the demo) while the UI keeps rendering; the main thread only
//! uploads finished tiles, a few per frame.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use glow::HasContext;
use imgui::TextureId;
use imgui_kit::map::TileGrid;

use crate::map_demo::{self, MapPage};

/// Which tile layer a request belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerId {
    Map,
    Geo,
}

struct Done {
    layer: LayerId,
    tile: [i32; 2],
    width: i32,
    height: i32,
    rgba: Vec<u8>,
}

pub struct TileLoader {
    requests: Sender<(LayerId, [i32; 2])>,
    done: Receiver<Done>,
    _worker: JoinHandle<()>,
}

impl TileLoader {
    /// Starts the worker. `asset_dir` is where `map/{x}_{y}.png` images live.
    pub fn start(asset_dir: PathBuf) -> Self {
        let (req_tx, req_rx) = channel::<(LayerId, [i32; 2])>();
        let (done_tx, done_rx) = channel::<Done>();
        let worker = thread::Builder::new()
            .name("tile-loader".into())
            .spawn(move || {
                // Ends when the loader (and so the request sender) is dropped.
                while let Ok((layer, tile)) = req_rx.recv() {
                    let (width, height, rgba) = match layer {
                        LayerId::Map => region_pixels(&asset_dir, tile),
                        LayerId::Geo => (256, 256, map_demo::geo_pixels(tile)),
                    };
                    if done_tx.send(Done { layer, tile, width, height, rgba }).is_err() {
                        break;
                    }
                }
            })
            .expect("spawn tile loader");
        Self { requests: req_tx, done: done_rx, _worker: worker }
    }

    /// Hands every tile the view marked pending to the worker.
    pub fn request_pending(&self, map: &mut MapPage) {
        for t in map.tiles.take_pending() {
            let _ = self.requests.send((LayerId::Map, t));
        }
        for t in map.geo.take_pending() {
            let _ = self.requests.send((LayerId::Geo, t));
        }
    }

    /// Uploads up to `max` finished tiles as GL textures and returns how many.
    pub fn upload_done(&self, gl: &glow::Context, map: &mut MapPage, max: usize) -> usize {
        let mut n = 0;
        while n < max {
            let Ok(d) = self.done.try_recv() else { break };
            let tex = upload_rgba(gl, d.width, d.height, &d.rgba);
            match d.layer {
                LayerId::Map => map.tiles.set(d.tile, tex),
                LayerId::Geo => map.geo.set(d.tile, tex),
            }
            n += 1;
        }
        n
    }
}

/// Pixels of a map region: `assets/map/{x}_{y}.png` (or `.jpg`) when present,
/// otherwise the synthetic terrain of the demo.
fn region_pixels(asset_dir: &std::path::Path, tile: [i32; 2]) -> (i32, i32, Vec<u8>) {
    let from_file = ["png", "jpg"].iter().find_map(|ext| {
        let path = asset_dir.join("map").join(format!("{}_{}.{ext}", tile[0], tile[1]));
        image::open(&path).ok().map(|img| img.to_rgba8())
    });
    match from_file {
        Some(img) => (img.width() as i32, img.height() as i32, img.into_raw()),
        None => (256, 256, map_demo::tile_pixels(tile)),
    }
}

/// Frees textures of tiles that left the view more than `max_age` frames ago.
pub fn evict(gl: &glow::Context, grids: [&mut TileGrid; 2], max_age: i32) {
    for grid in grids {
        for (_, tex) in grid.evict_unseen(max_age) {
            free_texture(gl, tex);
        }
    }
}

/// Creates a GL texture; imgui-glow-renderer's SimpleTextureMap uses the GL
/// name itself as the imgui texture id.
pub fn upload_rgba(gl: &glow::Context, w: i32, h: i32, pixels: &[u8]) -> TextureId {
    unsafe {
        let tex = gl.create_texture().expect("create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            w,
            h,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(pixels),
        );
        gl.bind_texture(glow::TEXTURE_2D, None);
        TextureId::new(tex.0.get() as usize)
    }
}

/// Frees a texture created by [`upload_rgba`].
pub fn free_texture(gl: &glow::Context, tex: TextureId) {
    if let Some(id) = std::num::NonZeroU32::new(tex.id() as u32) {
        unsafe { gl.delete_texture(glow::NativeTexture(id)) };
    }
}
