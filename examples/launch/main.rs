//! Runs the reference Launch screen in a real window. The screen itself
//! (`demo`), the widget gallery (`gallery`) and the Map tab (`map_demo`) are
//! modules of this example: the library ships no demo code.
//!
//!     cargo run --example launch
//!
//! Backend: winit + glutin (OpenGL) + imgui-glow-renderer. The OS window is
//! undecorated because the kit draws its own title bar; drag the bar to move
//! the window, the ─ □ × buttons minimize / maximize / close it.
//!
//! Frames are rendered on demand, not in a loop: after input, while an
//! animation runs (`kit.anim.animating`, `LaunchScreen::is_animating`) and at
//! a reduced rate when the window is not focused. Idle the process sleeps in
//! the event loop and uses no CPU; minimized it renders nothing.
//!
//! Map tiles are decoded on a worker thread (`tiles`), assets are found next
//! to the executable or in the checkout (`assets`), settings survive restarts
//! (`settings`), and a DPI change rebuilds the font atlas and the renderer.

use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

use glow::HasContext;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
mod assets;
mod demo;
mod gallery;
mod map_demo;
mod settings;
mod tiles;

use demo::{LaunchEvent, LaunchScreen};
use imgui::MouseButton;
use imgui_kit::{
    fonts::{self, FontFiles},
    theme,
    tokens::{color, size},
    widgets::TitleBarAction,
    Kit,
};
use settings::Settings;
use tiles::TileLoader;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const TITLE: &str = "PoEMulti · imgui_kit demo";
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

/// Frame period while focused and active (vsync caps it anyway).
const ACTIVE_PERIOD: Duration = Duration::from_millis(1000 / 60);
/// Frame period for self-animating content while the window is not focused.
const BACKGROUND_PERIOD: Duration = Duration::from_millis(1000 / 15);
/// Keep rendering this long after the last input so imgui settles
/// (hover, release, trickled key events, animations that start on the click).
const INPUT_GRACE: Duration = Duration::from_millis(500);
/// Finished tiles uploaded per frame (each is a 256 × 256 RGBA texture).
const UPLOADS_PER_FRAME: usize = 8;
/// Frames a tile may stay out of view before its texture is freed.
const TILE_MAX_AGE: i32 = 600;

/// The TTFs, embedded; also used to rebuild the atlas on a DPI change.
fn font_files() -> FontFiles<'static> {
    FontFiles {
        regular: include_bytes!("../../assets/JetBrainsMono-Regular.ttf"),
        bold: include_bytes!("../../assets/JetBrainsMono-Bold.ttf"),
        semibold: Some(include_bytes!("../../assets/JetBrainsMono-SemiBold.ttf")),
    }
}

fn main() {
    // Windows wakes WaitUntil timers every 15.6 ms by default; 1 ms gives the
    // on-demand loop real 60 fps while something animates.
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::Media::timeBeginPeriod(1);
    }

    let (event_loop, window, surface, context) = create_window();

    // imgui context + kit theme + kit fonts ---------------------------------
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    // imgui-sys ships without the Win32 default clipboard handlers, so
    // Ctrl+C / Ctrl+V in text fields and set_clipboard_text need a backend.
    match arboard::Clipboard::new() {
        Ok(c) => imgui.set_clipboard_backend(Clipboard(c)),
        Err(e) => eprintln!("clipboard unavailable: {e}"),
    }
    theme::apply_to(&mut imgui);
    // An app decision, not the kit's: the launcher never shrinks below this.
    imgui.style_mut().window_min_size = [960.0, 640.0];

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
    let scale = platform.hidpi_factor() as f32;

    let fonts = fonts::load(&mut imgui, font_files(), scale);
    imgui.io_mut().font_global_scale = 1.0 / scale;
    let mut kit = Kit::new(fonts);

    let gl = glow_context(&context);
    let mut renderer = imgui_glow_renderer::AutoRenderer::initialize(gl, &mut imgui)
        .expect("failed to create renderer");

    let mut screen = LaunchScreen::default();
    screen.restore(&kit, &Settings::load());
    let loader = TileLoader::start(assets::asset_dir());
    let mut pending_scale: Option<f32> = None;
    let mut last_frame = Instant::now();
    // render-on-demand state
    let mut last_input = Instant::now();
    let mut next_frame = Instant::now();
    let mut focused = true;
    let mut minimized = false;
    let mut animating = true;
    // IMGUI_KIT_FPS=1 prints frame-time stats once a second (perf checks).
    let mut stats = std::env::var_os("IMGUI_KIT_FPS").map(|_| FrameStats::default());
    let [r, g, b, _] = color::BG0;

    event_loop
        .run(move |event, target| match event {
            Event::AboutToWait => {
                // Decide whether another frame is needed and when.
                if minimized {
                    target.set_control_flow(ControlFlow::Wait);
                    return;
                }
                let now = Instant::now();
                let after_input = now.duration_since(last_input) < INPUT_GRACE;
                let period = if after_input || (animating && focused) {
                    Some(ACTIVE_PERIOD)
                } else if animating {
                    Some(BACKGROUND_PERIOD)
                } else {
                    None
                };
                match period {
                    Some(p) => {
                        // winit implements WaitUntil with a USER timer that
                        // fires every 15.6 ms on Windows (≈ 36 fps). Sleep
                        // ourselves instead (1 ms accurate after
                        // timeBeginPeriod) and keep the loop polling.
                        if now < next_frame {
                            std::thread::sleep(next_frame - now);
                        }
                        next_frame = Instant::now() + p;
                        window.request_redraw();
                        if let Some(s) = stats.as_mut() {
                            s.requests += 1;
                        }
                        target.set_control_flow(ControlFlow::Poll);
                    }
                    // Nothing moves: sleep until the OS sends an event.
                    None => target.set_control_flow(ControlFlow::Wait),
                }
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                // Delta time between rendered frames (not between event-loop
                // wake-ups: with on-demand rendering those are far more frequent).
                let now = Instant::now();
                let mut dt = now - last_frame;
                if dt > 2 * ACTIVE_PERIOD {
                    // The loop was asleep, not slow: do not let animations
                    // jump by the whole idle gap on their first frame.
                    dt = ACTIVE_PERIOD;
                }
                imgui.io_mut().update_delta_time(dt);
                last_frame = now;
                // DPI changed: new atlas at the new scale, and a new renderer
                // (the glow one bakes the font texture at creation).
                if let Some(s) = pending_scale.take() {
                    imgui.fonts().clear();
                    kit.fonts = fonts::load(&mut imgui, font_files(), s);
                    imgui.io_mut().font_global_scale = 1.0 / s;
                    renderer = imgui_glow_renderer::AutoRenderer::initialize(glow_context(&context), &mut imgui)
                        .expect("failed to recreate renderer");
                }
                platform.prepare_frame(imgui.io_mut(), &window).unwrap();
                unsafe {
                    let gl = renderer.gl_context();
                    gl.clear_color(r, g, b, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                let ui_start = Instant::now();
                let ui = imgui.frame();
                let display = ui.io().display_size;
                let events = screen.draw(ui, &kit, display);
                // Keep frames coming while something moves, and while a text
                // field has focus: imgui trickles queued key events one per
                // frame (a fast burst would lose its tail otherwise) and the
                // caret has to blink.
                animating = kit.anim.animating(ui) || screen.is_animating() || ui.io().want_text_input;

                // Map tiles: hand new requests to the worker, upload what it
                // finished, free textures that left the view a while ago, and
                // keep frames coming while anything is still in flight.
                let gl = renderer.gl_context();
                let map = &mut screen.gallery.map;
                loader.request_pending(map);
                let uploaded = loader.upload_done(gl, map, UPLOADS_PER_FRAME);
                tiles::evict(gl, [&mut map.tiles, &mut map.geo], TILE_MAX_AGE);
                let in_flight = map.tiles.counts().1 + map.geo.counts().1;
                if uploaded > 0 || in_flight > 0 {
                    animating = true;
                }

                // Empty title-bar area acts as the OS drag handle.
                let [_, my] = ui.io().mouse_pos;
                let drag = ui.is_mouse_clicked(MouseButton::Left)
                    && (0.0..size::BAR).contains(&my)
                    && !ui.is_any_item_hovered();

                platform.prepare_render(ui, &window);
                let draw_data = imgui.render();
                // UI build: widgets + draw lists, pure CPU, no GL calls.
                let ui_ms = ui_start.elapsed().as_secs_f32() * 1000.0;
                let (verts, idx) = (draw_data.total_vtx_count, draw_data.total_idx_count);
                renderer.render(draw_data).expect("error rendering imgui");
                surface.swap_buffers(&context).expect("failed to swap buffers");

                if let Some(s) = stats.as_mut() {
                    s.push(ui_ms, verts, idx, screen.tab, screen.gallery.map.view.zoom);
                }
                if drag {
                    let _ = window.drag_window();
                    // The OS runs the drag and eats the button release, so
                    // tell imgui ourselves or it keeps the button "held".
                    imgui.io_mut().add_mouse_button_event(MouseButton::Left, false);
                }
                for ev in events {
                    match ev {
                        LaunchEvent::Window(TitleBarAction::Close) => target.exit(),
                        LaunchEvent::Window(TitleBarAction::Minimize) => window.set_minimized(true),
                        LaunchEvent::Window(TitleBarAction::Maximize) => {
                            window.set_maximized(!window.is_maximized())
                        }
                        other => println!("event: {other:?}"),
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => target.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                minimized = new_size.width == 0 || new_size.height == 0;
                if !minimized {
                    surface.resize(
                        &context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
                last_input = Instant::now();
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                focused = f;
                last_input = Instant::now();
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { scale_factor, .. }, .. } => {
                pending_scale = Some(scale_factor as f32);
                last_input = Instant::now();
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            Event::LoopExiting => {
                if let Err(e) = screen.persist(&kit).save() {
                    eprintln!("settings not saved: {e}");
                }
            }
            Event::WindowEvent { .. } => {
                // Any other window event (mouse, keyboard, scale change, …)
                // is input: render for a short while after it.
                last_input = Instant::now();
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            event => platform.handle_event(imgui.io_mut(), &window, &event),
        })
        .expect("event loop error");
}

/// System clipboard for imgui (text fields, `set_clipboard_text`).
struct Clipboard(arboard::Clipboard);

impl imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.get_text().ok()
    }

    fn set(&mut self, value: &str) {
        if let Err(e) = self.0.set_text(value.to_owned()) {
            eprintln!("clipboard: {e}");
        }
    }
}

/// Per-second report of the UI build time (widgets + draw lists, no GL,
/// so no vsync wait inside) and the draw-list size.
#[derive(Default)]
struct FrameStats {
    samples: Vec<f32>,
    verts: i32,
    idx: i32,
    /// Redraws requested by the scheduler since the last report.
    requests: u32,
    since: Option<Instant>,
}

impl FrameStats {
    fn push(&mut self, ui_ms: f32, verts: i32, idx: i32, tab: usize, zoom: f32) {
        let since = *self.since.get_or_insert_with(Instant::now);
        self.samples.push(ui_ms);
        self.verts = self.verts.max(verts);
        self.idx = self.idx.max(idx);
        if since.elapsed().as_secs_f32() >= 1.0 {
            let n = self.samples.len() as f32;
            let avg = self.samples.iter().sum::<f32>() / n;
            let max = self.samples.iter().cloned().fold(0.0, f32::max);
            println!(
                "ui: tab {tab} zoom {zoom:.3} frames {} requests {} avg {avg:.2} ms max {max:.2} ms verts {} idx {}",
                self.samples.len(),
                self.requests,
                self.verts,
                self.idx
            );
            self.requests = 0;
            self.samples.clear();
            self.verts = 0;
            self.idx = 0;
            self.since = Some(Instant::now());
        }
    }
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

fn glow_context(context: &PossiblyCurrentContext) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function_cstr(|s| context.display().get_proc_address(s).cast())
    }
}
