//! Runs the reference Launch screen in a real window.
//!
//!     cargo run --example launch
//!
//! Backend: winit + glutin (OpenGL) + imgui-glow-renderer. The OS window is
//! undecorated because the kit draws its own title bar; drag the bar to move
//! the window, the ─ □ × buttons minimize / maximize / close it.

use std::{num::NonZeroU32, time::Instant};

use glow::HasContext;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use imgui::{MouseButton, TextureId};
use imgui_kit::{
    demo::{LaunchEvent, LaunchScreen},
    fonts::{self, FontFiles},
    map::TileGrid,
    map_demo, theme,
    tokens::{color, size},
    widgets::TitleBarAction,
};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const TITLE: &str = "PoEMulti · imgui_kit demo";
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

fn main() {
    let (event_loop, window, surface, context) = create_window();

    // imgui context + kit theme + kit fonts ---------------------------------
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    theme::apply_to(&mut imgui);

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
    let scale = platform.hidpi_factor() as f32;

    let fonts = fonts::load(
        &mut imgui,
        FontFiles {
            regular: include_bytes!("../assets/JetBrainsMono-Regular.ttf"),
            bold: include_bytes!("../assets/JetBrainsMono-Bold.ttf"),
            semibold: Some(include_bytes!("../assets/JetBrainsMono-SemiBold.ttf")),
        },
        scale,
    );
    imgui.io_mut().font_global_scale = 1.0 / scale;

    let gl = glow_context(&context);
    let mut renderer = imgui_glow_renderer::AutoRenderer::initialize(gl, &mut imgui)
        .expect("failed to create renderer");

    let mut screen = LaunchScreen::default();
    let mut last_frame = Instant::now();
    let [r, g, b, _] = color::BG0;

    event_loop
        .run(move |event, target| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::AboutToWait => {
                platform.prepare_frame(imgui.io_mut(), &window).unwrap();
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                unsafe {
                    let gl = renderer.gl_context();
                    gl.clear_color(r, g, b, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                let ui = imgui.frame();
                let display = ui.io().display_size;
                let ev = screen.draw(ui, &fonts, display);

                // Map tiles the view asked for this frame: decode + upload.
                load_pending_tiles(renderer.gl_context(), &mut screen.gallery.map.tiles);

                // Empty title-bar area acts as the OS drag handle.
                let [_, my] = ui.io().mouse_pos;
                let drag = ui.is_mouse_clicked(MouseButton::Left)
                    && (0.0..size::BAR).contains(&my)
                    && !ui.is_any_item_hovered();

                platform.prepare_render(ui, &window);
                let draw_data = imgui.render();
                renderer.render(draw_data).expect("error rendering imgui");
                surface.swap_buffers(&context).expect("failed to swap buffers");

                if drag {
                    let _ = window.drag_window();
                }
                match ev {
                    LaunchEvent::None => {}
                    LaunchEvent::Window(TitleBarAction::Close) => target.exit(),
                    LaunchEvent::Window(TitleBarAction::Minimize) => window.set_minimized(true),
                    LaunchEvent::Window(TitleBarAction::Maximize) => {
                        window.set_maximized(!window.is_maximized())
                    }
                    other => println!("event: {other:?}"),
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => target.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                if new_size.width > 0 && new_size.height > 0 {
                    surface.resize(
                        &context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            event => platform.handle_event(imgui.io_mut(), &window, &event),
        })
        .expect("event loop error");
}

fn create_window() -> (EventLoop<()>, Window, Surface<WindowSurface>, PossiblyCurrentContext) {
    let event_loop = EventLoop::new().unwrap();

    let window_builder = WindowBuilder::new()
        .with_title(TITLE)
        .with_decorations(false)
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));
    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("failed to create OpenGL window");
    let window = window.unwrap();

    let context_attribs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let context = unsafe {
        cfg.display()
            .create_context(&cfg, &context_attribs)
            .expect("failed to create OpenGL context")
    };

    let phys = window.inner_size();
    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            window.raw_window_handle(),
            NonZeroU32::new(phys.width.max(1)).unwrap(),
            NonZeroU32::new(phys.height.max(1)).unwrap(),
        );
    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &surface_attribs)
            .expect("failed to create OpenGL surface")
    };

    let context = context
        .make_current(&surface)
        .expect("failed to make OpenGL context current");
    // vsync
    let _ = surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));

    (event_loop, window, surface, context)
}

/// Loads every tile the map view marked pending. A region image is read
/// from `assets/map/{x}_{y}.png` (or `.jpg`) when present, otherwise the
/// synthetic terrain of the demo is rendered into a 256 × 256 texture.
fn load_pending_tiles(gl: &glow::Context, tiles: &mut TileGrid) {
    for tile in tiles.take_pending() {
        let from_file = ["png", "jpg"].iter().find_map(|ext| {
            let path = format!("assets/map/{}_{}.{ext}", tile[0], tile[1]);
            image::open(&path).ok().map(|img| img.to_rgba8())
        });
        let (w, h, pixels) = match from_file {
            Some(img) => (img.width() as i32, img.height() as i32, img.into_raw()),
            None => (256, 256, map_demo::tile_pixels(tile)),
        };
        tiles.set(tile, upload_rgba(gl, w, h, &pixels));
    }
}

/// Creates a GL texture; imgui-glow-renderer's SimpleTextureMap uses the GL
/// name itself as the imgui texture id.
fn upload_rgba(gl: &glow::Context, w: i32, h: i32, pixels: &[u8]) -> TextureId {
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

fn glow_context(context: &PossiblyCurrentContext) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function_cstr(|s| context.display().get_proc_address(s).cast())
    }
}
