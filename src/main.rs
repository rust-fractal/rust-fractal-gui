use std::sync::Arc;
// use std::time::Instant;

use parking_lot::Mutex;

use druid::{widget::prelude::*};

use druid::{AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KbKey, FileDialogOptions, FileSpec, Data, Lens, MenuDesc};
use druid::piet::{ImageFormat, InterpolationMode};


use druid::commands::{
    OPEN_FILE,
    SAVE_FILE_AS,
    SHOW_OPEN_PANEL,
    SHOW_SAVE_PANEL
};

use rust_fractal::{renderer::FractalRenderer, util::data_export::DataExport};
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long, string_to_extended};

use config::{Config, File};

use std::thread;
use std::sync::mpsc;

use atomic_counter::{AtomicCounter, RelaxedCounter};

mod ui;
pub mod lens;
mod saving;
pub mod custom;
pub mod render_thread;
pub mod formatters;
pub mod commands;
pub mod theme;

use crate::commands::*;
use crate::theme::*;

use render_thread::testing_renderer;

struct FractalWidget {
    buffer: Vec<u8>,
    image_width: usize,
    image_height: usize,
    save_type: usize
}

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    updated: usize,
    temporary_width: i64,
    temporary_height: i64,
    temporary_real: String,
    temporary_imag: String,
    temporary_zoom_mantissa: f64,
    temporary_zoom_exponent: i64,
    temporary_zoom_string: String,
    temporary_iterations: i64,
    temporary_rotation: f64,
    temporary_order: i64,
    temporary_palette_source: String,
    temporary_location_source: String,
    temporary_iteration_division: f64,
    temporary_iteration_offset: f64,
    temporary_progress: f64,
    temporary_stage: usize,
    temporary_time: usize,
    temporary_min_valid_iterations: usize,
    temporary_max_valid_iterations: usize,
    temporary_display_glitches: bool,
    temporary_glitch_tolerance: f64,
    temporary_glitch_percentage: f64,
    temporary_iteration_interval: i64,
    temporary_experimental: bool,
    temporary_probe_sampling: i64,
    temporary_jitter: bool,
    temporary_jitter_factor: f64,
    temporary_auto_adjust_iterations: bool,
    temporary_remove_center: bool,
    renderer: Arc<Mutex<FractalRenderer>>,
    settings: Arc<Mutex<Config>>,
    sender: Arc<Mutex<mpsc::Sender<usize>>>,
    stop_flag: Arc<RelaxedCounter>,
    repeat_flag: Arc<RelaxedCounter>,
    buffer: Arc<Mutex<DataExport>>,
    need_full_rerender: bool,
    zoom_out_enabled: bool,
    show_settings: bool
}

impl Widget<FractalData> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        ctx.request_focus();
        // println!("{:?}", event);

        match event {
            Event::WindowConnected => {
                let settings = data.settings.lock();

                data.temporary_width = settings.get_int("image_width").unwrap();
                data.temporary_height = settings.get_int("image_height").unwrap();

                let sender = data.sender.lock();
                // sender.send(String::from("reset_renderer_full")).unwrap();
                sender.send(THREAD_RESET_RENDERER_FULL).unwrap();

                data.updated += 1;
            }
            Event::MouseDown(e) => {
                // If the rendering has not completed, stop
                if data.temporary_stage != 0 {
                    return;
                }

                let mut settings = data.settings.lock();
                let mut renderer = data.renderer.lock();

                // For a mousedown event we only check the left and right buttons
                if e.button == MouseButton::Left || e.button == MouseButton::Right {
                    // Zoom in, use the mouse position
                    if e.button == MouseButton::Left {
                        let size = ctx.size().to_rect();

                        let i = e.pos.x * renderer.image_width as f64 / size.width();
                        let j = e.pos.y * renderer.image_height as f64 / size.height();
    
                        let cos_rotate = renderer.rotate.cos();
                        let sin_rotate = renderer.rotate.sin();
    
                        let delta_pixel =  4.0 / ((renderer.image_height - 1) as f64 * renderer.zoom.mantissa);
                        let delta_top_left = get_delta_top_left(delta_pixel, renderer.image_width, renderer.image_height, cos_rotate, sin_rotate);
    
                        let element = ComplexFixed::new(
                            i * delta_pixel * cos_rotate - j * delta_pixel * sin_rotate + delta_top_left.re, 
                            i * delta_pixel * sin_rotate + j * delta_pixel * cos_rotate + delta_top_left.im
                        );

                        let element = ComplexExtended::new(element, -renderer.zoom.exponent);
                        let mut zoom = renderer.zoom;
                    
                        zoom.mantissa *= 2.0;
                        zoom.reduce();

                        let mut location = renderer.center_reference.c.clone();
                        let precision = location.real().prec();

                        let temp = FloatArbitrary::with_val(precision, element.exponent).exp2();
                        let temp2 = FloatArbitrary::with_val(precision, element.mantissa.re);
                        let temp3 = FloatArbitrary::with_val(precision, element.mantissa.im);

                        *location.mut_real() += &temp2 * &temp;
                        *location.mut_imag() += &temp3 * &temp;

                        data.temporary_zoom_string = extended_to_string_long(zoom);

                        // Set the overrides for the current location
                        settings.set("real", location.real().to_string()).unwrap();
                        settings.set("imag", location.imag().to_string()).unwrap();
                        settings.set("zoom", data.temporary_zoom_string.clone()).unwrap();

                        data.temporary_real = settings.get_str("real").unwrap();
                        data.temporary_imag = settings.get_str("imag").unwrap();

                        let temp: Vec<&str> = data.temporary_zoom_string.split('E').collect();
                        data.temporary_zoom_mantissa = temp[0].parse::<f64>().unwrap();
                        data.temporary_zoom_exponent = temp[1].parse::<i64>().unwrap();

                        renderer.adjust_iterations();

                        settings.set("iterations", renderer.maximum_iteration as i64).unwrap();
                        data.temporary_iterations = renderer.maximum_iteration as i64;

                        ctx.submit_command(RESET_RENDERER_FULL);
                    } else {
                        ctx.submit_command(MULTIPLY_ZOOM.with(0.5));
                    }
                }
            },
            Event::KeyUp(e) => {
                // Shortcut keys
                if e.key == KbKey::Character("D".to_string()) || e.key == KbKey::Character("d".to_string()) {
                    ctx.submit_command(TOGGLE_DERIVATIVE);
                }

                if e.key == KbKey::Character("Z".to_string()) || e.key == KbKey::Character("z".to_string()) {
                    ctx.submit_command(MULTIPLY_ZOOM.with(2.0));
                }

                if e.key == KbKey::Character("O".to_string()) || e.key == KbKey::Character("o".to_string()) {
                    ctx.submit_command(OPEN_LOCATION);
                }

                if e.key == KbKey::Character("N".to_string()) || e.key == KbKey::Character("n".to_string()) {
                    ctx.submit_command(NATIVE_SIZE);
                }

                if e.key == KbKey::Character("T".to_string()) || e.key == KbKey::Character("t".to_string()) {
                    ctx.submit_command(MULTIPLY_SIZE.with(0.5));
                }

                if e.key == KbKey::Character("Y".to_string()) || e.key == KbKey::Character("y".to_string()) {
                    ctx.submit_command(MULTIPLY_SIZE.with(2.0));
                }

                if e.key == KbKey::Character("R".to_string()) || e.key == KbKey::Character("r".to_string()) {
                    let settings = data.settings.lock();
                    let new_rotate = (settings.get_float("rotate").unwrap() + 15.0) % 360.0;

                    ctx.submit_command(SET_ROTATION.with(new_rotate));
                }

                if e.mods.ctrl() && (e.key == KbKey::Character("S".to_string()) || e.key == KbKey::Character("s".to_string())) {
                    ctx.submit_command(SAVE_ALL);
                }
            },
            Event::Command(command) => {
                // println!("{:?}", command);

                if command.is(UPDATE_PALETTE) {
                    return;
                }

                if command.is(STOP_RENDERING) {
                    if data.temporary_stage != 0 || data.zoom_out_enabled {
                        data.stop_flag.inc();
                    }

                    // if the renderer was stopped during SA / reference
                    if data.temporary_stage == 1 || data.temporary_stage == 2 {
                        data.stop_flag.inc();
                    }

                    if data.zoom_out_enabled {
                        data.repeat_flag.inc();
                    }

                    // println!("stop {} {}", data.zoom_out_enabled, data.repeat_flag.get());

                    data.zoom_out_enabled = false;

                    return;
                }

                if command.is(REPAINT) {
                    if data.stop_flag.get() >= 2 {
                        // use wrapping to reset to zero
                        data.need_full_rerender = true;
                    } else {
                        data.need_full_rerender = false;
                    }

                    data.stop_flag.add(usize::max_value() - data.stop_flag.get() + 1);

                    data.updated += 1;

                    // let start = Instant::now();

                    // TODO need to update image width and height
                    let buffer = data.buffer.lock();

                    self.buffer = buffer.rgb.clone();

                    if self.image_width != buffer.image_width || self.image_height != buffer.image_height {
                        self.image_width = buffer.image_width;
                        self.image_height = buffer.image_height;
                        ctx.request_layout();
                    }

                    // let time = start.elapsed().as_millis() as usize;

                    // println!("buffer copy: {}ms", time);

                    ctx.request_paint();

                    return;
                }

                if let Some((stage, progress, time, min_valid_iterations, max_valid_iterations)) = command.get(UPDATE_PROGRESS) {
                    data.show_settings = false;
                    data.temporary_progress = *progress;
                    data.temporary_stage = *stage;
                    data.temporary_time = *time;
                    data.temporary_min_valid_iterations = *min_valid_iterations;
                    data.temporary_max_valid_iterations = *max_valid_iterations;
                    return;
                }

                // If the rendering has not completed, stop
                if data.temporary_stage != 0 {
                    return;
                }

                let mut settings = data.settings.lock();
                let mut renderer = data.renderer.lock();

                if let Some(factor) = command.get(MULTIPLY_SIZE) {
                    let new_width = settings.get_int("image_width").unwrap() as f64 * factor;
                    let new_height = settings.get_int("image_height").unwrap() as f64 * factor;

                    ctx.submit_command(SET_SIZE.with((new_width as i64, new_height as i64)));
                    return;
                }

                if command.is(NATIVE_SIZE) {
                    let window_width = settings.get_float("window_width").unwrap();
                    let window_height = settings.get_float("window_height").unwrap();

                    ctx.submit_command(SET_SIZE.with((window_width as i64, window_height as i64)));
                    return;
                }

                if let Some(dimensions) = command.get(SET_SIZE) {
                    if dimensions.0 as usize == renderer.image_width && dimensions.1 as usize == renderer.image_height {
                        return;
                    }

                    settings.set("image_width", dimensions.0 as i64).unwrap();
                    settings.set("image_height", dimensions.1 as i64).unwrap();

                    renderer.image_width = dimensions.0 as usize;
                    renderer.image_height = dimensions.1 as usize;

                    ctx.submit_command(RESET_RENDERER_FAST);
                    return;
                }

                // At the moment, if the reference has already been done at a higher iteration number we just set the data export
                // iteration number to less, rather than actually reducing the iteration level
                if let Some(iterations) = command.get(SET_ITERATIONS) {
                    if *iterations as usize == renderer.data_export.lock().maximum_iteration {
                        return;
                    }

                    settings.set("iterations", *iterations).unwrap();
                    data.temporary_iterations = *iterations;

                    if *iterations as usize <= renderer.maximum_iteration {
                        renderer.data_export.lock().maximum_iteration = data.temporary_iterations as usize;
                        renderer.data_export.lock().regenerate();

                        ctx.submit_command(REPAINT);
                        return;
                    }

                    ctx.submit_command(RESET_RENDERER_FULL);
                    return;
                }

                // Handles setting the advanced options
                if command.is(SET_ADVANCED_OPTIONS) {
                    println!("{} {} {}", data.temporary_remove_center, renderer.remove_centre, renderer.data_export.lock().centre_removed);

                    // These options require the entire renderer to be refreshed
                    if renderer.center_reference.data_storage_interval != data.temporary_iteration_interval as usize ||
                        renderer.center_reference.glitch_tolerance != data.temporary_glitch_tolerance {
                        if data.temporary_iteration_interval < 1 {
                            data.temporary_iteration_interval = 1;
                        }

                        if data.temporary_glitch_tolerance < 0.0 {
                            data.temporary_glitch_tolerance = 0.0;
                        }


                        ctx.submit_command(RESET_RENDERER_FULL);

                        println!("interval or glitch tolerance changed");
                    } else if renderer.series_approximation.order != data.temporary_order as usize ||
                        renderer.series_approximation.probe_sampling != data.temporary_probe_sampling as usize ||
                        renderer.series_approximation.experimental != data.temporary_experimental {
                        // apply limits to the values 

                        if (data.temporary_order as usize) > 128 {
                            data.temporary_order = 128;
                        } else if (data.temporary_order as usize) < 4 {
                            data.temporary_order = 4;
                        }

                        if (data.temporary_probe_sampling as usize) > 128 {
                            data.temporary_probe_sampling = 128;
                        } else if (data.temporary_probe_sampling as usize) < 2 {
                            data.temporary_probe_sampling = 2;
                        }

                        renderer.progress.reset_series_approximation();

                        // renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                        ctx.submit_command(RESET_RENDERER_FAST);

                        println!("order or probe sampling or experimental changed");
                    } else if renderer.glitch_percentage != data.temporary_glitch_percentage || 
                        renderer.jitter != data.temporary_jitter ||
                        renderer.remove_centre != data.temporary_remove_center ||
                        (renderer.jitter && renderer.jitter_factor != data.temporary_jitter_factor) {

                        if data.temporary_glitch_percentage > 100.0 {
                            data.temporary_glitch_percentage = 100.0;
                        }
    
                        if data.temporary_glitch_percentage < 0.0 {
                            data.temporary_glitch_percentage = 0.0;
                        }

                        if data.temporary_jitter_factor > 100.0 {
                            data.temporary_glitch_percentage = 100.0;
                        }
    
                        if data.temporary_jitter_factor < 0.0 {
                            data.temporary_glitch_percentage = 0.0;
                        }

                        if data.temporary_remove_center {
                            renderer.data_export.lock().clear_buffers();
                        }

                        ctx.submit_command(RESET_RENDERER_FAST);

                        println!("glitch percentage or jitter or remove centre changed");
                    } else if renderer.data_export.lock().display_glitches != data.temporary_display_glitches {
                        renderer.data_export.lock().display_glitches = data.temporary_display_glitches;
                        renderer.data_export.lock().regenerate();
                        ctx.submit_command(REPAINT);

                        println!("display glitches changed");
                    } else if renderer.auto_adjust_iterations != data.temporary_auto_adjust_iterations {
                        println!("auto adjust iterations changed");
                    } else {
                        println!("nothing changed");
                        return;
                    }

                    // set all the config to be updated
                    settings.set("data_storage_interval", data.temporary_iteration_interval).unwrap();
                    settings.set("glitch_tolerance", data.temporary_glitch_tolerance).unwrap();
                    settings.set("approximation_order", data.temporary_order).unwrap();
                    settings.set("probe_sampling", data.temporary_probe_sampling).unwrap();
                    settings.set("experimental", data.temporary_experimental).unwrap();
                    settings.set("glitch_percentage", data.temporary_glitch_percentage).unwrap();
                    settings.set("jitter", data.temporary_jitter).unwrap();
                    settings.set("jitter_factor", data.temporary_jitter_factor).unwrap();
                    settings.set("remove_centre", data.temporary_remove_center).unwrap();
                    settings.set("display_glitches", data.temporary_display_glitches).unwrap();
                    settings.set("auto_adjust_iterations", data.temporary_auto_adjust_iterations).unwrap();

                    renderer.center_reference.data_storage_interval = data.temporary_iteration_interval as usize;
                    renderer.center_reference.glitch_tolerance = data.temporary_glitch_tolerance;

                    renderer.series_approximation.order = data.temporary_order as usize;
                    renderer.series_approximation.probe_sampling = data.temporary_probe_sampling as usize;
                    renderer.series_approximation.experimental = data.temporary_experimental;

                    renderer.glitch_percentage = data.temporary_glitch_percentage;
                    renderer.jitter = data.temporary_jitter;
                    renderer.jitter_factor = data.temporary_jitter_factor;

                    println!("setting to {}", data.temporary_remove_center);
                    renderer.remove_centre = data.temporary_remove_center;
                    // renderer.data_export.lock().centre_removed = false;

                    renderer.data_export.lock().display_glitches = data.temporary_display_glitches;

                    renderer.auto_adjust_iterations = data.temporary_auto_adjust_iterations;
                    return;
                }

                if command.is(SET_LOCATION) {
                    let current_real = settings.get_str("real").unwrap();
                    let current_imag = settings.get_str("imag").unwrap();
                    let current_zoom = settings.get_str("zoom").unwrap();

                    let current_iterations = settings.get_int("iterations").unwrap();
                    let current_rotation = settings.get_float("rotate").unwrap();

                    data.temporary_zoom_string = format!("{}E{}", data.temporary_zoom_mantissa, data.temporary_zoom_exponent);

                    if current_real == data.temporary_real && current_imag == data.temporary_imag {
                        // Check if the zoom has decreased or is near to the current level
                        if current_zoom.to_uppercase() == data.temporary_zoom_string.to_uppercase() {
                            // nothing has changed
                            if current_rotation == data.temporary_rotation && current_iterations == data.temporary_iterations {
                                // println!("nothing");
                                return;
                            }

                            // iterations changed
                            if current_iterations == data.temporary_iterations {
                                // println!("rotation");
                                ctx.submit_command(SET_ROTATION.with(data.temporary_rotation));
                                return;
                            }

                            if current_rotation == data.temporary_rotation {
                                // println!("iterations");
                                ctx.submit_command(SET_ITERATIONS.with(data.temporary_iterations));
                                return;
                            }

                            // println!("rotation & iterations");

                            settings.set("iterations", data.temporary_iterations).unwrap();

                            if (data.temporary_iterations as usize) < renderer.maximum_iteration {
                                // TODO needs to make it so that pixels are only iterated to the right level
                                renderer.maximum_iteration = data.temporary_iterations as usize;
                                ctx.submit_command(SET_ROTATION.with(data.temporary_rotation));
                                return;
                            }
                        } else {
                            // Zoom has changed, and need to rerender depending on if the zoom has changed too much

                            let current_exponent = renderer.center_reference.zoom.exponent;
                            let new_zoom = string_to_extended(&data.temporary_zoom_string.to_uppercase());

                            if new_zoom.exponent <= current_exponent {
                                // println!("zoom decreased");
                                renderer.zoom = new_zoom;
                                settings.set("zoom", data.temporary_zoom_string.clone()).unwrap();
                                renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                                ctx.submit_command(RESET_RENDERER_FAST);
                                return;
                            }
                        }
                    }

                    // println!("location changed / zoom increased / iterations increased and rotation");

                    settings.set("real", data.temporary_real.clone()).unwrap();
                    settings.set("imag", data.temporary_imag.clone()).unwrap();
                    settings.set("zoom",  data.temporary_zoom_string.clone()).unwrap();
                    settings.set("rotate", data.temporary_rotation.clone()).unwrap();
                    settings.set("iterations", data.temporary_iterations.clone()).unwrap();

                    ctx.submit_command(RESET_RENDERER_FULL);
                    return;
                }

                if let Some(factor) = command.get(MULTIPLY_ZOOM) {
                    renderer.zoom.mantissa *= factor;
                    renderer.zoom.reduce();

                    data.temporary_zoom_string = extended_to_string_long(renderer.zoom);
                    settings.set("zoom", data.temporary_zoom_string.clone()).unwrap();
                    
                    let temp: Vec<&str> = data.temporary_zoom_string.split('E').collect();
                    data.temporary_zoom_mantissa = temp[0].parse::<f64>().unwrap();
                    data.temporary_zoom_exponent = temp[1].parse::<i64>().unwrap();

                    data.need_full_rerender &= renderer.adjust_iterations();

                    settings.set("iterations", renderer.maximum_iteration as i64).unwrap();
                    data.temporary_iterations = renderer.maximum_iteration as i64;

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    // TODO properly set the maximum iterations
                    ctx.submit_command(RESET_RENDERER_FAST);
                    return;
                }

                if command.is(ZOOM_OUT) {
                    // renderer.remaining_frames = 2;

                    data.zoom_out_enabled = true;
                    data.repeat_flag.add(usize::max_value() - data.repeat_flag.get() + 1);

                    // println!("start zoom out: {}", data.repeat_flag.get());

                    ctx.submit_command(MULTIPLY_ZOOM.with(0.5));

                    return;
                }

                if command.is(ZOOM_OUT_OPTIMISED) {
                    renderer.remaining_frames = 2;
                    renderer.remove_centre = true;

                    renderer.data_export.lock().centre_removed = false;
                    renderer.data_export.lock().clear_buffers();

                    data.temporary_remove_center = true;
                    settings.set("remove_centre", true).unwrap();

                    data.zoom_out_enabled = true;
                    data.repeat_flag.add(usize::max_value() - data.repeat_flag.get() + 1);

                    ctx.submit_command(MULTIPLY_ZOOM.with(0.5));
                    return;
                }

                if command.is(TOGGLE_DERIVATIVE) {
                    let current_derivative = settings.get_bool("analytic_derivative").unwrap();
                    settings.set("analytic_derivative", !current_derivative).unwrap();

                    renderer.data_export.lock().analytic_derivative = !current_derivative;

                    // We have already computed the iterations and analytic derivatives
                    if renderer.analytic_derivative {
                        renderer.data_export.lock().regenerate();
                        ctx.submit_command(REPAINT);
                    } else {
                        renderer.analytic_derivative = true;
                        ctx.submit_command(RESET_RENDERER_FAST);
                    };

                    return;
                }

                if let Some(rotation) = command.get(SET_ROTATION) {
                    let new_rotate = (rotation % 360.0 + 360.0) % 360.0;

                    settings.set("rotate", new_rotate).unwrap();
                    data.temporary_rotation = new_rotate;

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    renderer.rotate = new_rotate.to_radians();

                    ctx.submit_command(RESET_RENDERER_FAST);
                    return;
                }

                if command.is(SET_OFFSET_DIVISION) {
                    let current_division = settings.get_float("iteration_division").unwrap();
                    let current_offset = settings.get_float("palette_offset").unwrap();

                    let new_division = data.temporary_iteration_division;
                    let new_offset = data.temporary_iteration_offset % renderer.data_export.lock().palette.len() as f64;

                    // println!("{} {} {}", data.temporary_iteration_offset, new_offset, new_division);

                    if current_division == new_division && current_offset == new_offset {
                        return;
                    }

                    data.temporary_iteration_division = new_division;
                    data.temporary_iteration_offset = new_offset;

                    settings.set("iteration_division", new_division).unwrap();
                    settings.set("palette_offset", new_offset).unwrap();

                    renderer.data_export.lock().change_palette(None, new_division as f32, new_offset as f32);
                    renderer.data_export.lock().regenerate();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();

                    ctx.submit_command(REPAINT);

                    return;
                }

                if command.is(RESET_RENDERER_FAST) {
                    // renderer.maximum_iteration = renderer.data_export.lock().maximum_iteration;

                    if data.need_full_rerender {
                        // println!("needs full rerender");
                        ctx.submit_command(RESET_RENDERER_FULL);
                        return;
                    }

                    let sender = data.sender.lock();
                    sender.send(THREAD_RESET_RENDERER_FAST).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    return;
                }

                if command.is(RESET_RENDERER_FULL) {
                    let sender = data.sender.lock();
                    sender.send(THREAD_RESET_RENDERER_FULL).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    return;
                }

                if command.is(OPEN_LOCATION) {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let open_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    ctx.submit_command(SHOW_OPEN_PANEL.with(open_dialog_options));
                    return;
                }

                if command.is(SAVE_LOCATION) {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let save_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    self.save_type = 0;

                    ctx.submit_command(SHOW_SAVE_PANEL.with(save_dialog_options));
                    return;
                }

                if command.is(SAVE_ALL) {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let save_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    self.save_type = 1;

                    ctx.submit_command(SHOW_SAVE_PANEL.with(save_dialog_options));
                    return;
                }

                if command.is(SAVE_IMAGE) {
                    let png = FileSpec::new("Portable Network Graphics", &["png"]);
                    let jpg = FileSpec::new("JPEG", &["jpg"]);

                    let save_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![png, jpg]);

                    self.save_type = 2;

                    ctx.submit_command(SHOW_SAVE_PANEL.with(save_dialog_options));
                    return;
                }

                if let Some(file_info) = command.get(OPEN_FILE) {
                    settings.set("export", "gui").unwrap();

                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    let file_name = file_info.path().file_name().unwrap().to_str().unwrap().split(".").next().unwrap();

                    let mut reset_renderer = false;
                    let mut quick_reset = false;

                    match new_settings.get_str("real") {
                        Ok(real) => {
                            settings.set("real", real.clone()).unwrap();
                            data.temporary_real = real;
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("imag") {
                        Ok(imag) => {
                            settings.set("imag", imag.clone()).unwrap();
                            data.temporary_imag = imag;
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("zoom") {
                        Ok(zoom) => {
                            settings.set("zoom", zoom.clone()).unwrap();
                            data.temporary_zoom_string = zoom.to_uppercase();

                            let temp: Vec<&str> = data.temporary_zoom_string.split('E').collect();
                            data.temporary_zoom_mantissa = temp[0].parse::<f64>().unwrap();
                            data.temporary_zoom_exponent = temp[1].parse::<i64>().unwrap();

                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_int("iterations") {
                        Ok(iterations) => {
                            settings.set("iterations", iterations.clone()).unwrap();
                            data.temporary_iterations = iterations;
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_float("rotate") {
                        Ok(rotate) => {
                            settings.set("rotate", rotate.clone()).unwrap();
                            data.temporary_rotation = rotate;
                            reset_renderer = true;
                        }
                        Err(_) => {
                            settings.set("rotate", 0.0).unwrap();
                            data.temporary_rotation = 0.0;
                        }
                    }

                    match new_settings.get_int("image_width") {
                        Ok(width) => {
                            data.temporary_width = width;
                            settings.set("image_width", width.clone()).unwrap();
                            quick_reset = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_int("image_height") {
                        Ok(height) => {
                            data.temporary_height = height;
                            settings.set("image_height", height.clone()).unwrap();
                            quick_reset = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_int("approximation_order") {
                        Ok(order) => {
                            data.temporary_order = order;
                            settings.set("approximation_order", order.clone()).unwrap();
                            quick_reset = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_bool("analytic_derivative") {
                        Ok(analytic_derivative) => {
                            renderer.data_export.lock().analytic_derivative = analytic_derivative;
                            settings.set("analytic_derivative", analytic_derivative.clone()).unwrap();
                            quick_reset = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_array("palette") {
                        Ok(colour_values) => {
                            // Only reset these if the palette is defined
                            match new_settings.get_float("iteration_division") {
                                Ok(iteration_division) => {
                                    settings.set("iteration_division", iteration_division).unwrap();
                                    data.temporary_iteration_division = iteration_division;
                                }
                                Err(_) => {
                                    settings.set("iteration_division", 1.0).unwrap();
                                    data.temporary_iteration_division = 1.0;
                                }
                            }
        
                            match new_settings.get_float("palette_offset") {
                                Ok(palette_offset) => {
                                    settings.set("palette_offset", palette_offset).unwrap();
                                    data.temporary_iteration_offset = palette_offset;
                                }
                                Err(_) => {
                                    settings.set("palette_offset", 0.0).unwrap();
                                    data.temporary_iteration_offset = 0.0;
                                }
                            }

                            settings.set("palette", colour_values.clone()).unwrap();
                            ctx.submit_command(UPDATE_PALETTE);

                            let palette = colour_values.chunks_exact(3).map(|value| {
                                // We assume the palette is in BGR rather than RGB
                                (value[2].clone().into_int().unwrap() as u8, 
                                    value[1].clone().into_int().unwrap() as u8, 
                                    value[0].clone().into_int().unwrap() as u8)
                            }).collect::<Vec<(u8, u8, u8)>>();

                            renderer.data_export.lock().change_palette(
                                Some(palette),
                                settings.get_float("iteration_division").unwrap() as f32,
                                settings.get_float("palette_offset").unwrap() as f32
                            );

                            data.temporary_palette_source = file_name.to_string();

                            if !reset_renderer || !quick_reset {
                                renderer.data_export.lock().regenerate();
                                ctx.submit_command(REPAINT);
                            }
                        }
                        Err(_) => {}
                    }

                    settings.merge(new_settings).unwrap();

                    if reset_renderer {
                        data.temporary_location_source = file_name.to_string();
                        // println!("calling full reset");
                        ctx.submit_command(RESET_RENDERER_FULL);
                    } else if quick_reset {
                        data.temporary_location_source = file_name.to_string();
                        // println!("calling full reset");
                        ctx.submit_command(RESET_RENDERER_FAST);
                    }

                    return;
                }

                if let Some(file_info) = command.get(SAVE_FILE_AS) {
                    match self.save_type {
                        0 => {
                            let real = settings.get_str("real").unwrap();
                            let imag = settings.get_str("imag").unwrap();
                            let zoom = settings.get_str("zoom").unwrap();
                            let iterations = settings.get_int("iterations").unwrap();
                            let rotate = settings.get_float("rotate").unwrap();

                            let output = format!("real = \"{}\"\nimag = \"{}\"\nzoom = \"{}\"\niterations = {}\nrotate = {}", real, imag, zoom, iterations.to_string(), rotate.to_string());

                            if let Err(e) = std::fs::write(file_info.path(), output) {
                                println!("Error writing file: {}", e);
                            }
                        },
                        1 => {
                            let real = settings.get_str("real").unwrap();
                            let imag = settings.get_str("imag").unwrap();
                            let zoom = settings.get_str("zoom").unwrap();
                            let iterations = settings.get_int("iterations").unwrap();
                            let rotate = settings.get_float("rotate").unwrap();

                            let image_width = settings.get_int("image_width").unwrap();
                            let image_height = settings.get_int("image_height").unwrap();
                            let glitch_percentage = settings.get_float("glitch_percentage").unwrap();
                            let approximation_order = settings.get_int("approximation_order").unwrap();
                            let analytic_derivative = settings.get_bool("analytic_derivative").unwrap();

                            let palette = renderer.data_export.lock().palette.clone().into_iter().flat_map(|seq| {
                                // BGR format
                                vec![seq.2, seq.1, seq.0]
                            }).collect::<Vec<u8>>();
                            let iteration_division = settings.get_float("iteration_division").unwrap();
                            let palette_offset = settings.get_float("palette_offset").unwrap();

                            let output = format!(
                                "version = \"{}\"\n\nreal = \"{}\"\nimag = \"{}\"\nzoom = \"{}\"\niterations = {}\nrotate = {}\n\nimage_width = {}\nimage_height = {}\nglitch_percentage = {}\napproximation_order = {}\nanalytic_derivative = {}\nframes = 1\nframe_offset = 0\nzoom_scale = 2.0\ndisplay_glitches = false\nauto_adjust_iterations = true\nremove_centre = false\nglitch_tolerance = 1.4e-6\nprobe_sampling = 15\ndata_storage_interval = 100\nvalid_iteration_frame_multiplier = 0.10\nvalid_iteration_probe_multiplier = 0.01\nexperimental = true\njitter = false\nexport = \"png\"\n\npalette = {:?}\niteration_division = {}\npalette_offset = {}", 
                                env!("CARGO_PKG_VERSION"),
                                real, 
                                imag, 
                                zoom, 
                                iterations.to_string(), 
                                rotate.to_string(),
                                image_width,
                                image_height,
                                glitch_percentage,
                                approximation_order,
                                analytic_derivative,
                                palette,
                                iteration_division,
                                palette_offset);

                            if let Err(e) = std::fs::write(file_info.path(), output) {
                                println!("Error writing file: {}", e);
                            }
                        },
                        2 => {
                            renderer.data_export.lock().save_colour(file_info.path().to_str().unwrap());
                        },
                        _ => {}
                    }

                    return;
                }
            },
            _ => {}
        }
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FractalData, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &FractalData, _data: &FractalData, _env: &Env) {
        // println!("update called");
        return;
    }

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FractalData, _env: &Env) -> Size {
        // println!("layout called");
        let mut test = bc.max();

        let mut settings = data.settings.lock();

        settings.set("window_width", test.width).unwrap();
        settings.set("window_height", test.height).unwrap();

        let aspect_image = self.image_width as f64 / self.image_height as f64;
        let aspect_constraints = test.width / test.height;

        // If the aspect ratio is greater than the size, we limit based on the width
        if aspect_image >= aspect_constraints {
            test.height = test.width / aspect_image;
        } else {
            test.width = test.height * aspect_image;
        }

        test
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &FractalData, _env: &Env) {
        if self.image_width * self.image_height > 0 {
            // let start = Instant::now();

            let size = ctx.size().to_rect();

            let image = ctx
                .make_image(self.image_width, self.image_height, &self.buffer, ImageFormat::Rgb)
                .unwrap();

            if self.image_width > size.width() as usize || self.image_height > size.height() as usize {
                ctx.draw_image(&image, size, InterpolationMode::Bilinear);
            } else {
                ctx.draw_image(&image, size, InterpolationMode::NearestNeighbor);
            };

            // let time = start.elapsed().as_millis() as usize;

            // println!("paint: {}ms", time);
        }
    }

    fn id(&self) -> Option<WidgetId> {
        None
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub fn main() {
    // Setup the default settings. These are stored in start.toml file
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    let zoom_string = settings.get_str("zoom").unwrap();
    let temp: Vec<&str> = zoom_string.split('E').collect();

    let window_title = Box::leak(format!("rust-fractal {}", env!("CARGO_PKG_VERSION")).into_boxed_str());

    let window = WindowDesc::new(ui::ui_builder()).title(
        LocalizedString::new(window_title),
    ).window_size((1388.0, 827.0)).resizable(true).menu(make_menu());

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, reciever) = mpsc::channel();

    let shared_settings = Arc::new(Mutex::new(settings.clone()));
    let shared_renderer = Arc::new(Mutex::new(FractalRenderer::new(settings.clone())));
    let shared_stop_flag = Arc::new(RelaxedCounter::new(0));
    let shared_repeat_flag = Arc::new(RelaxedCounter::new(1));

    let thread_settings = shared_settings.clone();
    let thread_renderer = shared_renderer.clone();
    let thread_stop_flag = shared_stop_flag.clone();
    let thread_repeat_flag = shared_repeat_flag.clone();

    let buffer = shared_renderer.lock().data_export.clone();

    thread::spawn(move || testing_renderer(event_sink, reciever, thread_settings, thread_renderer, thread_stop_flag, thread_repeat_flag));

    launcher
        .configure_env(|env, _| configure_env(env))
        .launch(FractalData {
            updated: 0,
            temporary_width: settings.get_int("image_width").unwrap(),
            temporary_height: settings.get_int("image_height").unwrap(),
            temporary_real: settings.get_str("real").unwrap(),
            temporary_imag: settings.get_str("imag").unwrap(),
            temporary_zoom_mantissa: temp[0].parse::<f64>().unwrap(),
            temporary_zoom_exponent: temp[1].parse::<i64>().unwrap(),
            temporary_zoom_string: zoom_string,
            temporary_iterations: settings.get_int("iterations").unwrap(),
            temporary_rotation: settings.get_float("rotate").unwrap(),
            temporary_order: settings.get_int("approximation_order").unwrap(),
            temporary_palette_source: "default".to_string(),
            temporary_location_source: "default".to_string(),
            temporary_iteration_division: settings.get_float("iteration_division").unwrap(),
            temporary_iteration_offset: settings.get_float("palette_offset").unwrap(),
            temporary_progress: 0.0,
            temporary_stage: 1,
            temporary_time: 0,
            temporary_min_valid_iterations: 1,
            temporary_max_valid_iterations: 1,
            temporary_display_glitches: settings.get_bool("display_glitches").unwrap(),
            temporary_glitch_tolerance: settings.get_float("glitch_tolerance").unwrap(),
            temporary_glitch_percentage: settings.get_float("glitch_percentage").unwrap(),
            temporary_iteration_interval: settings.get_int("data_storage_interval").unwrap(),
            temporary_experimental: settings.get_bool("experimental").unwrap(),
            temporary_probe_sampling: settings.get_int("probe_sampling").unwrap(),
            temporary_jitter: settings.get_bool("jitter").unwrap(),
            temporary_jitter_factor: settings.get_float("jitter_factor").unwrap(),
            temporary_auto_adjust_iterations: settings.get_bool("auto_adjust_iterations").unwrap(),
            temporary_remove_center: settings.get_bool("remove_centre").unwrap(),
            renderer: shared_renderer,
            settings: shared_settings,
            sender: Arc::new(Mutex::new(sender)),
            stop_flag: shared_stop_flag,
            repeat_flag: shared_repeat_flag,
            buffer,
            need_full_rerender: false,
            zoom_out_enabled: false,
            show_settings: true,
        })
        .expect("launch failed");
}

#[allow(unused_assignments, unused_mut)]
fn make_menu<T: Data>() -> MenuDesc<T> {
    let mut base = MenuDesc::empty();
    // #[cfg(target_os = "macos")]
    // {
    //     base = base.append(druid::platform_menus::mac::application::default())
    // }
    // #[cfg(any(target_os = "windows", target_os = "linux"))]
    // {
    //     base = base.append(druid::platform_menus::win::file::default());
    // }
    base.append(
        MenuDesc::new(LocalizedString::new("common-menu-edit-menu"))
            .append(druid::platform_menus::common::cut())
            .append(druid::platform_menus::common::copy())
            .append(druid::platform_menus::common::paste()),
    )
}