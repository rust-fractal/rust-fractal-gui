use std::sync::Arc;
// use std::time::Instant;
use parking_lot::Mutex;

use druid::{AppLauncher, LocalizedString, WindowDesc};

use rust_fractal::{renderer::FractalRenderer};
use rust_fractal::util::{extended_to_string_long, string_to_extended};
use rust_fractal::util::data_export::ColoringType;

use config::{Config, File};

use std::thread;
use std::sync::mpsc;
use std::sync::atomic::AtomicBool;

use rust_fractal_gui::theme::*;
use rust_fractal_gui::render_thread::testing_renderer;
use rust_fractal_gui::ui;
use rust_fractal_gui::widgets::FractalData;

pub fn main() {
    // Setup the default settings. These are stored in start.toml file
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    if settings.get_bool("show_output").unwrap() {
        println!(" {:<15}| {:<15}| {:<15}| {:<6}| {:<15}| {:<15}| {:<15}| {:<6}| {:<15}", "Zoom", "Approx [ms]", "Skipped [it]", "Order", "Maximum [it]", "Iteration [ms]", "Correct [ms]", "Ref", "Frame [ms]");
    };

    let shared_settings = Arc::new(Mutex::new(settings.clone()));
    let shared_renderer = Arc::new(Mutex::new(FractalRenderer::new(settings.clone())));
    let shared_stop_flag = Arc::new(AtomicBool::new(false));
    let shared_repeat_flag = Arc::new(AtomicBool::new(false));

    let thread_settings = shared_settings.clone();
    let thread_renderer = shared_renderer.clone();
    let thread_stop_flag = shared_stop_flag.clone();
    let thread_repeat_flag = shared_repeat_flag.clone();

    let buffer = shared_renderer.lock().data_export.clone();

    let window = WindowDesc::new(ui::window_main(shared_renderer.clone())).title(
        LocalizedString::new("rust-fractal"),
    ).window_size((1392.0, 830.0)).resizable(true).menu(ui::make_menu);

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, reciever) = mpsc::channel();

    let mut center_reference_zoom = string_to_extended(&settings.get_str("zoom").unwrap());
    center_reference_zoom.exponent += 40;

    thread::spawn(move || testing_renderer(event_sink, reciever, thread_settings, thread_renderer, thread_stop_flag, thread_repeat_flag));

    launcher
        .configure_env(|env, _| configure_env(env))
        .launch(FractalData {
            image_width: settings.get_int("image_width").unwrap(),
            image_height: settings.get_int("image_height").unwrap(),
            real: settings.get_str("real").unwrap(),
            imag: settings.get_str("imag").unwrap(),
            zoom: settings.get_str("zoom").unwrap(),
            root_zoom: "1E0".to_string(),
            iteration_limit: settings.get_int("iterations").unwrap(),
            rotation: settings.get_float("rotate").unwrap(),
            order: settings.get_int("approximation_order").unwrap(),
            period: 0,
            palette_source: "default".to_string(),
            palette_cyclic: settings.get_bool("palette_cyclic").unwrap(),
            palette_iteration_span: settings.get_float("palette_iteration_span").unwrap(),
            palette_offset: settings.get_float("palette_offset").unwrap(),
            rendering_progress: 0.0,
            root_progress: 1.0,
            rendering_stage: 1,
            rendering_time: 0,
            root_iteration: 64,
            root_stage: 0,
            min_valid_iterations: 1,
            max_valid_iterations: 1,
            min_iterations: 1,
            max_iterations: 1,
            display_glitches: settings.get_bool("display_glitches").unwrap(),
            glitch_tolerance: settings.get_float("glitch_tolerance").unwrap(),
            glitch_percentage: settings.get_float("glitch_percentage").unwrap(),
            iteration_interval: settings.get_int("data_storage_interval").unwrap(),
            series_approximation_tiled: settings.get_bool("series_approximation_tiled").unwrap(),
            series_approximation_enabled: settings.get_bool("series_approximation_enabled").unwrap(),
            probe_sampling: settings.get_int("probe_sampling").unwrap(),
            jitter: settings.get_bool("jitter").unwrap(),
            jitter_factor: settings.get_float("jitter_factor").unwrap(),
            auto_adjust_iterations: settings.get_bool("auto_adjust_iterations").unwrap(),
            remove_centre: settings.get_bool("remove_centre").unwrap(),
            renderer: shared_renderer,
            settings: shared_settings,
            sender: Arc::new(Mutex::new(sender)),
            stop_flag: shared_stop_flag,
            repeat_flag: shared_repeat_flag,
            buffer,
            need_full_rerender: false,
            zoom_out_enabled: false,
            pixel_pos: [0, 0],
            pixel_iterations: 1,
            pixel_smooth: 0.0,
            pixel_rgb: Arc::new(Mutex::new(vec![0u8; 255 * 3])),
            coloring_type: ColoringType::SmoothIteration,
            mouse_mode: 0,
            current_tab: 0,
            zoom_scale_factor: settings.get_float("zoom_scale").unwrap(),
            root_zoom_factor: 0.5,
            center_reference_zoom: extended_to_string_long(center_reference_zoom),
            reference_count: 1,
            stripe_scale: settings.get_float("stripe_scale").unwrap() as f32,
            distance_transition: settings.get_float("distance_transition").unwrap() as f32,
            distance_color: settings.get_bool("distance_color").unwrap(),
            lighting: settings.get_bool("lighting").unwrap(),
            lighting_direction: settings.get_float("lighting_direction").unwrap(),
            lighting_azimuth: settings.get_float("lighting_azimuth").unwrap(),
            lighting_opacity: settings.get_float("lighting_opacity").unwrap(),
            lighting_ambient: settings.get_float("lighting_ambient").unwrap(),
            lighting_diffuse: settings.get_float("lighting_diffuse").unwrap(),
            lighting_specular: settings.get_float("lighting_specular").unwrap(),
            lighting_shininess: settings.get_int("lighting_shininess").unwrap(),
        })
        .expect("launch failed");
}