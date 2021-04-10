use std::sync::Arc;
// use std::time::Instant;

use parking_lot::Mutex;

use druid::{widget::prelude::*};

use druid::{AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KbKey, FileDialogOptions, FileSpec, Data, Lens, Rect};
use druid::piet::{ImageFormat, InterpolationMode, D2DRenderContext, Color};

use druid::kurbo::Circle;


use druid::commands::{
    OPEN_FILE,
    SAVE_FILE_AS,
    SHOW_OPEN_PANEL,
    SHOW_SAVE_PANEL
};

use rust_fractal::{renderer::FractalRenderer};
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long, string_to_extended, linear_interpolation_between_zoom, data_export::DataExport, data_export::ColoringType};
use rust_fractal::math::BoxPeriod;

use config::{Config, File};

use std::thread;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cmp::min;

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

struct FractalWidget<'a> {
    image_width: usize,
    image_height: usize,
    save_type: usize,
    newton_pos1: (f64, f64),
    newton_pos2: (f64, f64),
    root_pos_start: (f64, f64),
    root_pos_current: (f64, f64),
    cached_image: Option<<D2DRenderContext<'a> as RenderContext>::Image>,
    needs_buffer_refresh: bool,
    show_selecting_box: bool,
    renderer_zoom: FloatExtended,
    renderer_rotate: (f64, f64)
}

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    image_width: i64,
    image_height: i64,
    real: String,
    imag: String,
    zoom: String,
    root_zoom: String,
    iteration_limit: i64,
    rotation: f64,
    order: i64,
    period: usize,
    palette_source: String,
    iteration_span: f64,
    iteration_offset: f64,
    #[data(same_fn = "PartialEq::eq")]
    coloring_type: ColoringType,
    rendering_progress: f64,
    root_progress: f64,
    rendering_stage: usize,
    rendering_time: usize,
    root_iteration: usize,
    root_stage: usize,
    min_valid_iterations: usize,
    max_valid_iterations: usize,
    min_iterations: usize,
    max_iterations: usize,
    display_glitches: bool,
    glitch_tolerance: f64,
    glitch_percentage: f64,
    iteration_interval: i64,
    experimental: bool,
    probe_sampling: i64,
    jitter: bool,
    jitter_factor: f64,
    auto_adjust_iterations: bool,
    remove_centre: bool,
    renderer: Arc<Mutex<FractalRenderer>>,
    settings: Arc<Mutex<Config>>,
    sender: Arc<Mutex<mpsc::Sender<usize>>>,
    stop_flag: Arc<AtomicBool>,
    repeat_flag: Arc<AtomicBool>,
    buffer: Arc<Mutex<DataExport>>,
    need_full_rerender: bool,
    zoom_out_enabled: bool,
    pixel_pos: [u32; 2],
    pixel_iterations: u32,
    pixel_smooth: f32,
    pixel_rgb: Arc<Mutex<Vec<u8>>>,
    mouse_mode: usize,
    current_tab: usize,
    zoom_scale_factor: f64,
    root_zoom_factor: f64,
    center_reference_zoom: String,
}

impl<'a> Widget<FractalData> for FractalWidget<'a> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        ctx.request_focus();
        // println!("{:?}", event);

        match event {
            Event::WindowConnected => {
                let settings = data.settings.lock();

                data.image_width = settings.get_int("image_width").unwrap();
                data.image_height = settings.get_int("image_height").unwrap();

                data.sender.lock().send(THREAD_RESET_RENDERER_FULL).unwrap();
            }
            // TODO this section sometimes blocks. Needs to be fixed
            Event::MouseMove(e) => {
                if data.mouse_mode != 0 {
                    self.newton_pos2 = (e.pos.x, e.pos.y);
                    
                    let top_left = (self.newton_pos1.0.min(self.newton_pos2.0), self.newton_pos1.1.min(self.newton_pos2.1));
                    let bottom_right = (self.newton_pos1.0.max(self.newton_pos2.0), self.newton_pos1.1.max(self.newton_pos2.1));

                    self.root_pos_current = (0.5 * (top_left.0 + bottom_right.0), 0.5 * (top_left.1 + bottom_right.1));

                    ctx.request_paint();
                }
            }
            //     if data.stage == 0 {
            //         let size = ctx.size().to_rect();

            //         let i = e.pos.x * data.image_width as f64 / size.width();
            //         let j = e.pos.y * data.image_height as f64 / size.height();

            //         let k = j as usize * data.image_width as usize + i as usize;

            //         data.pixel_pos[0] = i as u32;
            //         data.pixel_pos[1] = j as u32;

            //         let data_export = data.buffer.lock();

            //         data.pixel_iterations = data_export.iterations[k];
            //         data.pixel_smooth = if data_export.smooth[k] <= 1.0 {
            //             data_export.smooth[k]
            //         } else {
            //             0.0
            //         };

            //         drop(data_export);

            //         let mut pixel_buffer = data.pixel_rgb.lock();

            //         if i as i64 > 15 && i as i64 + 15 < data.image_width && j as i64 > 15 && j as i64 + 15 < data.image_height {
            //             let mut temp = 0;
            //             for n in (j as usize - 7)..=(j as usize + 7) {
            //                 for m in (i as usize - 7)..=(i as usize + 7) {
            //                     let o = 3 * (n * data.image_width as usize + m);

            //                     pixel_buffer[temp] = self.buffer[o];
            //                     pixel_buffer[temp + 1] = self.buffer[o + 1];
            //                     pixel_buffer[temp + 2] = self.buffer[o + 2];

            //                     temp += 3;
            //                 }
            //             }
            //         };

            //         drop(pixel_buffer);

            //         ctx.submit_command(UPDATE_PIXEL_INFORMATION);
            //     }
            // }
            Event::MouseDown(e) => {
                // If the rendering has not completed, stop
                if data.rendering_stage != 0 || data.root_stage == 1 {
                    return;
                }

                // Zoom in, use the mouse position
                if e.button == MouseButton::Left {
                    if data.mouse_mode == 0 {
                        let mut settings = data.settings.lock();
                        let mut renderer = data.renderer.lock();
    
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
                    
                        zoom.mantissa *= data.zoom_scale_factor;
                        zoom.reduce();
    
                        let mut location = renderer.center_reference.c.clone();
                        let precision = location.real().prec();
    
                        let temp = FloatArbitrary::with_val(precision, element.exponent).exp2();
                        let temp2 = FloatArbitrary::with_val(precision, element.mantissa.re);
                        let temp3 = FloatArbitrary::with_val(precision, element.mantissa.im);
    
                        *location.mut_real() += &temp2 * &temp;
                        *location.mut_imag() += &temp3 * &temp;
    
                        data.zoom = extended_to_string_long(zoom);
    
                        // Set the overrides for the current location
                        settings.set("real", location.real().to_string()).unwrap();
                        settings.set("imag", location.imag().to_string()).unwrap();
                        settings.set("zoom", data.zoom.clone()).unwrap();
    
                        data.real = settings.get_str("real").unwrap();
                        data.imag = settings.get_str("imag").unwrap();
    
                        renderer.adjust_iterations();
    
                        settings.set("iterations", renderer.maximum_iteration as i64).unwrap();
                        data.iteration_limit = renderer.maximum_iteration as i64;
    
                        ctx.submit_command(RESET_RENDERER_FULL);
                    } else {
                        self.show_selecting_box = true;
                        self.newton_pos1 = (e.pos.x, e.pos.y);
                    }
                }

                if e.button == MouseButton::Right {
                    ctx.submit_command(MULTIPLY_ZOOM.with(1.0 / data.zoom_scale_factor));
                }
            },
            Event::MouseUp(e) => {
                if e.button == MouseButton::Left && data.mouse_mode != 0 {
                    data.mouse_mode = 0;
                    self.newton_pos2 = (e.pos.x, e.pos.y);

                    ctx.submit_command(CALCULATE_ROOT);
                }
            }
            Event::KeyUp(e) => {
                // Shortcut keys
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

                if let Some(period) = command.get(SET_PERIOD) {
                    data.period = *period;

                    return;
                }

                if let Some(root_zoom) = command.get(ROOT_FINDING_COMPLETE) {
                    self.show_selecting_box = false;

                    data.root_progress = 1.0;
                    data.root_iteration = 64;

                    if let Some(root_zoom) = root_zoom {
                        data.root_zoom = extended_to_string_long(*root_zoom);
                        data.root_stage = 0;
                    } else {
                        ctx.request_paint();
                        data.root_stage = 2;
                    }

                    return;
                }

                if command.is(STOP_ROOT_FINDING) {
                    if data.root_stage != 0 {
                        data.stop_flag.store(true, Ordering::SeqCst);
                    }

                    return;
                }

                if command.is(STOP_RENDERING) {
                    if data.rendering_stage != 0 || data.zoom_out_enabled {
                        data.stop_flag.store(true, Ordering::SeqCst);
                    }

                    // if the renderer was stopped during SA / reference
                    data.need_full_rerender = data.rendering_stage == 1 || data.rendering_stage == 2;

                    if data.zoom_out_enabled {
                        data.repeat_flag.store(false, Ordering::SeqCst);
                    }

                    data.zoom_out_enabled = false;

                    return;
                }

                if command.is(REPAINT) {
                    let buffer = data.buffer.lock();

                    if self.image_width != buffer.image_width || self.image_height != buffer.image_height {
                        self.image_width = buffer.image_width;
                        self.image_height = buffer.image_height;
                        ctx.request_layout();
                    }

                    self.needs_buffer_refresh = true;
                    ctx.request_paint();

                    return;
                }

                if let Some((iteration, progress, position)) = command.get(UPDATE_ROOT_PROGRESS) {
                    data.root_iteration = *iteration;
                    data.root_progress = *progress as f64 / data.period as f64;

                    let delta_pixel =  4.0 / ((data.image_height - 1) as f64 * self.renderer_zoom.mantissa);

                    let size = ctx.size().to_rect();

                    let difference_fixed = position.mantissa * 2.0f64.powi(position.exponent + self.renderer_zoom.exponent) / delta_pixel;
                    let difference_real = (self.renderer_rotate.0 * difference_fixed.re + self.renderer_rotate.1 * difference_fixed.im) * size.width() / data.image_width as f64;
                    let difference_imag = (-self.renderer_rotate.1 * difference_fixed.re + self.renderer_rotate.0 * difference_fixed.im) * size.height() / data.image_height as f64;

                    self.root_pos_current = (self.root_pos_start.0 + difference_real, self.root_pos_start.1 + difference_imag);

                    return;
                }

                if let Some((stage, progress, time, min_valid_iterations, max_valid_iterations)) = command.get(UPDATE_RENDERING_PROGRESS) {
                    data.rendering_progress = *progress;
                    data.rendering_stage = *stage;
                    data.rendering_time = *time;

                    if *stage >= 3 || *stage == 0 {
                        data.min_valid_iterations = *min_valid_iterations;
                        data.max_valid_iterations = *max_valid_iterations;
                    }

                    if *stage == 0 {
                        let temp = *data.buffer.lock().iterations.iter().min().unwrap() as usize;

                        data.min_iterations = if temp != 0xFFFFFFFF {
                            temp
                        } else {
                            1
                        };

                        data.max_iterations = min(*data.buffer.lock().iterations.iter().max().unwrap() as usize, data.iteration_limit as usize);
                    }
                    
                    return;
                }

                // If the rendering / root finding has not completed, stop
                if data.rendering_stage != 0 || data.root_stage == 1 {
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
                    data.image_width = settings.get_float("window_width").unwrap() as i64;
                    data.image_height = settings.get_float("window_height").unwrap() as i64;
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

                    renderer.total_pixels = renderer.image_width * renderer.image_height;

                    if renderer.remove_centre {
                        let temp = 1.0 / renderer.zoom_scale_factor;

                        // Add one to avoid rescaling artifacts
                        let val1 = (renderer.image_width as f64 * temp).ceil() as usize - 1;
                        let val2 = (renderer.image_height as f64 * temp).ceil() as usize - 1;
                
                        renderer.total_pixels -= val1 * val2;
                    }

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
                    data.iteration_limit = *iterations;

                    if *iterations as usize <= renderer.maximum_iteration {
                        renderer.data_export.lock().maximum_iteration = data.iteration_limit as usize;
                        renderer.data_export.lock().regenerate();

                        ctx.submit_command(REPAINT);
                        return;
                    }

                    ctx.submit_command(RESET_RENDERER_FULL);
                    return;
                }

                // Handles setting the advanced options
                if command.is(SET_ADVANCED_OPTIONS) {
                    // println!("{} {} {}", data.remove_centre, renderer.remove_centre, renderer.data_export.lock().centre_removed);

                    // These options require the entire renderer to be refreshed
                    if renderer.center_reference.data_storage_interval != data.iteration_interval as usize ||
                        renderer.center_reference.glitch_tolerance != data.glitch_tolerance {
                        if data.iteration_interval < 1 {
                            data.iteration_interval = 1;
                        }

                        if data.glitch_tolerance < 0.0 {
                            data.glitch_tolerance = 0.0;
                        }


                        ctx.submit_command(RESET_RENDERER_FULL);

                        // println!("interval or glitch tolerance changed");
                    } else if renderer.series_approximation.order != data.order as usize ||
                        renderer.series_approximation.probe_sampling != data.probe_sampling as usize ||
                        renderer.series_approximation.experimental != data.experimental {
                        // apply limits to the values 

                        if (data.order as usize) > 128 {
                            data.order = 128;
                        } else if (data.order as usize) < 4 {
                            data.order = 4;
                        }

                        if (data.probe_sampling as usize) > 128 {
                            data.probe_sampling = 128;
                        } else if (data.probe_sampling as usize) < 2 {
                            data.probe_sampling = 2;
                        }

                        renderer.progress.reset_series_approximation();

                        // renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                        ctx.submit_command(RESET_RENDERER_FAST);

                        // println!("order or probe sampling or experimental changed");
                    } else if renderer.glitch_percentage != data.glitch_percentage || 
                        renderer.jitter != data.jitter ||
                        renderer.remove_centre != data.remove_centre ||
                        (renderer.jitter && renderer.jitter_factor != data.jitter_factor) {

                        if data.glitch_percentage > 100.0 {
                            data.glitch_percentage = 100.0;
                        }
    
                        if data.glitch_percentage < 0.0 {
                            data.glitch_percentage = 0.0;
                        }

                        if data.jitter_factor > 100.0 {
                            data.glitch_percentage = 100.0;
                        }
    
                        if data.jitter_factor < 0.0 {
                            data.glitch_percentage = 0.0;
                        }

                        if data.remove_centre {
                            renderer.data_export.lock().clear_buffers();
                        }

                        ctx.submit_command(RESET_RENDERER_FAST);

                        // println!("glitch percentage or jitter or remove centre changed");
                    } else if renderer.data_export.lock().display_glitches != data.display_glitches {
                        renderer.data_export.lock().display_glitches = data.display_glitches;
                        renderer.data_export.lock().regenerate();
                        ctx.submit_command(REPAINT);

                        // println!("display glitches changed");
                    } else if renderer.auto_adjust_iterations != data.auto_adjust_iterations {
                        // println!("auto adjust iterations changed");
                    } else {
                        // println!("nothing changed");
                        return;
                    }

                    // set all the config to be updated
                    settings.set("data_storage_interval", data.iteration_interval).unwrap();
                    settings.set("glitch_tolerance", data.glitch_tolerance).unwrap();
                    settings.set("approximation_order", data.order).unwrap();
                    settings.set("probe_sampling", data.probe_sampling).unwrap();
                    settings.set("experimental", data.experimental).unwrap();
                    settings.set("glitch_percentage", data.glitch_percentage).unwrap();
                    settings.set("jitter", data.jitter).unwrap();
                    settings.set("jitter_factor", data.jitter_factor).unwrap();
                    settings.set("remove_centre", data.remove_centre).unwrap();
                    settings.set("display_glitches", data.display_glitches).unwrap();
                    settings.set("auto_adjust_iterations", data.auto_adjust_iterations).unwrap();

                    renderer.center_reference.data_storage_interval = data.iteration_interval as usize;
                    renderer.center_reference.glitch_tolerance = data.glitch_tolerance;

                    renderer.series_approximation.order = data.order as usize;
                    renderer.series_approximation.probe_sampling = data.probe_sampling as usize;
                    renderer.series_approximation.experimental = data.experimental;

                    renderer.glitch_percentage = data.glitch_percentage;
                    renderer.jitter = data.jitter;
                    renderer.jitter_factor = data.jitter_factor;

                    renderer.zoom_scale_factor = data.zoom_scale_factor;

                    // println!("setting to {}", data.remove_centre);
                    renderer.remove_centre = data.remove_centre;
                    // renderer.data_export.lock().centre_removed = false;

                    renderer.data_export.lock().display_glitches = data.display_glitches;

                    renderer.auto_adjust_iterations = data.auto_adjust_iterations;
                    return;
                }

                if command.is(SET_LOCATION) {
                    let current_real = settings.get_str("real").unwrap();
                    let current_imag = settings.get_str("imag").unwrap();
                    let current_zoom = settings.get_str("zoom").unwrap();

                    let current_iterations = settings.get_int("iterations").unwrap();
                    let current_rotation = settings.get_float("rotate").unwrap();

                    if current_real == data.real && current_imag == data.imag {
                        // Check if the zoom has decreased or is near to the current level
                        if current_zoom.to_uppercase() == data.zoom.to_uppercase() {
                            // nothing has changed
                            if current_rotation == data.rotation && current_iterations == data.iteration_limit {
                                // println!("nothing");
                                return;
                            }

                            // iterations changed
                            if current_iterations == data.iteration_limit {
                                // println!("rotation");
                                ctx.submit_command(SET_ROTATION.with(data.rotation));
                                return;
                            }

                            if current_rotation == data.rotation {
                                // println!("iterations");
                                ctx.submit_command(SET_ITERATIONS.with(data.iteration_limit));
                                return;
                            }

                            // println!("rotation & iterations");

                            settings.set("iterations", data.iteration_limit).unwrap();

                            if (data.iteration_limit as usize) < renderer.maximum_iteration {
                                // TODO needs to make it so that pixels are only iterated to the right level
                                renderer.maximum_iteration = data.iteration_limit as usize;
                                ctx.submit_command(SET_ROTATION.with(data.rotation));
                                return;
                            }
                        } else {
                            // Zoom has changed, and need to rerender depending on if the zoom has changed too much

                            let current_exponent = renderer.center_reference.zoom.exponent;
                            let new_zoom = string_to_extended(&data.zoom);

                            if new_zoom.exponent <= current_exponent {
                                // println!("zoom decreased");
                                renderer.zoom = new_zoom;
                                settings.set("zoom", data.zoom.clone()).unwrap();
                                renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                                ctx.submit_command(RESET_RENDERER_FAST);
                                return;
                            }
                        }
                    }

                    // println!("location changed / zoom increased / iterations increased and rotation");

                    settings.set("real", data.real.clone()).unwrap();
                    settings.set("imag", data.imag.clone()).unwrap();
                    settings.set("zoom",  data.zoom.clone()).unwrap();
                    settings.set("rotate", data.rotation).unwrap();
                    settings.set("iterations", data.iteration_limit).unwrap();

                    ctx.submit_command(RESET_RENDERER_FULL);
                    return;
                }

                // TODO maybe enable the iterations and rotation parts
                if command.is(REVERT_LOCATION) {
                    data.real = settings.get_str("real").unwrap();
                    data.imag = settings.get_str("imag").unwrap();
                    data.zoom = settings.get_str("zoom").unwrap();
                    data.iteration_limit = settings.get_int("iterations").unwrap();

                    // let current_rotation = settings.get_float("rotate").unwrap();
                }

                if let Some(factor) = command.get(MULTIPLY_PATTERN) {
                    let new_zoom = linear_interpolation_between_zoom(renderer.zoom, string_to_extended(&data.root_zoom), *factor);

                    renderer.zoom = new_zoom;

                    data.zoom = extended_to_string_long(renderer.zoom);
                    settings.set("zoom", data.zoom.clone()).unwrap();

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                    // if there is no zoom in
                    if string_to_extended(&data.zoom) > string_to_extended(&data.center_reference_zoom) {
                        // println!("resetting fast");
                        // println!("zoom pattern need rerender");
                        data.need_full_rerender = true;
                    };

                    ctx.submit_command(RESET_RENDERER_FAST);

                    return;
                }

                if let Some(factor) = command.get(MULTIPLY_ZOOM) {
                    renderer.zoom.mantissa *= factor;
                    renderer.zoom.reduce();

                    data.zoom = extended_to_string_long(renderer.zoom);
                    settings.set("zoom", data.zoom.clone()).unwrap();

                    data.need_full_rerender &= renderer.adjust_iterations();

                    settings.set("iterations", renderer.maximum_iteration as i64).unwrap();
                    data.iteration_limit = renderer.maximum_iteration as i64;

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    
                    if string_to_extended(&data.zoom) > string_to_extended(&data.center_reference_zoom) {
                        // println!("zoom quick need rerender");
                        data.need_full_rerender = true;
                    };

                    ctx.submit_command(RESET_RENDERER_FAST);

                    return;
                }

                if command.is(ZOOM_OUT) {
                    data.zoom_out_enabled = true;
                    data.repeat_flag.store(true, Ordering::SeqCst);

                    ctx.submit_command(MULTIPLY_ZOOM.with(0.5));

                    return;
                }

                if command.is(ZOOM_OUT_OPTIMISED) {
                    renderer.remaining_frames = 2;
                    renderer.remove_centre = true;

                    renderer.data_export.lock().centre_removed = false;
                    renderer.data_export.lock().clear_buffers();

                    data.remove_centre = true;
                    settings.set("remove_centre", true).unwrap();

                    data.zoom_out_enabled = true;
                    data.repeat_flag.store(true, Ordering::SeqCst);

                    ctx.submit_command(MULTIPLY_ZOOM.with(0.5));
                    return;
                }

                if let Some(coloring_method) = command.get(SET_COLORING_METHOD) {
                    if coloring_method != &data.coloring_type {
                        renderer.data_export.lock().coloring_type = *coloring_method;

                        match coloring_method {
                            ColoringType::SmoothIteration => {
                                settings.set("analytic_derivative", false).unwrap();
                                settings.set("step_iteration", false).unwrap();

                                renderer.data_export.lock().regenerate();
                                ctx.submit_command(REPAINT);
                            }
                            ColoringType::StepIteration => {
                                settings.set("analytic_derivative", false).unwrap();
                                settings.set("step_iteration", true).unwrap();

                                renderer.data_export.lock().regenerate();
                                ctx.submit_command(REPAINT);
                            }
                            ColoringType::Distance => {
                                settings.set("analytic_derivative", true).unwrap();

                                if renderer.analytic_derivative {
                                    renderer.data_export.lock().regenerate();
                                    ctx.submit_command(REPAINT);
                                } else {
                                    renderer.analytic_derivative = true;
                                    ctx.submit_command(RESET_RENDERER_FAST);
                                };
                            }
                        }
                    }

                    data.coloring_type = *coloring_method;
                    return;
                }

                if let Some(rotation) = command.get(SET_ROTATION) {
                    let new_rotate = (rotation % 360.0 + 360.0) % 360.0;

                    settings.set("rotate", new_rotate).unwrap();
                    data.rotation = new_rotate;

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    renderer.rotate = new_rotate.to_radians();

                    ctx.submit_command(RESET_RENDERER_FAST);
                    return;
                }

                if command.is(SET_OFFSET_SPAN) {
                    let current_division = settings.get_float("iteration_division").unwrap();
                    let current_offset = settings.get_float("palette_offset").unwrap();

                    let new_division = data.iteration_span;
                    let new_offset = data.iteration_offset;

                    // println!("{} {} {}", data.temporary_iteration_offset, new_offset, new_division);

                    if current_division == new_division && current_offset == new_offset {
                        return;
                    }

                    data.iteration_span = new_division;
                    data.iteration_offset = new_offset;

                    settings.set("iteration_division", new_division).unwrap();
                    settings.set("palette_offset", new_offset).unwrap();

                    renderer.data_export.lock().change_palette(None, new_division as f32, new_offset as f32);
                    renderer.data_export.lock().regenerate();

                    data.image_width = settings.get_int("image_width").unwrap();
                    data.image_height = settings.get_int("image_height").unwrap();

                    ctx.submit_command(REPAINT);

                    return;
                }

                if command.is(RESET_RENDERER_FAST) {
                    // renderer.maximum_iteration = renderer.data_export.lock().maximum_iteration;

                    if data.need_full_rerender {
                        // println!("needs full rerender");
                        data.need_full_rerender = false;
                        ctx.submit_command(RESET_RENDERER_FULL);
                        return;
                    }

                    let sender = data.sender.lock();
                    sender.send(THREAD_RESET_RENDERER_FAST).unwrap();

                    data.image_width = settings.get_int("image_width").unwrap();
                    data.image_height = settings.get_int("image_height").unwrap();
                    data.min_valid_iterations = 1;
                    data.max_valid_iterations = 1;
                    data.min_iterations = 1;
                    data.max_iterations = 1;

                    return;
                }

                if command.is(RESET_RENDERER_FULL) {
                    let sender = data.sender.lock();
                    sender.send(THREAD_RESET_RENDERER_FULL).unwrap();

                    data.image_width = settings.get_int("image_width").unwrap();
                    data.image_height = settings.get_int("image_height").unwrap();
                    data.min_valid_iterations = 1;
                    data.max_valid_iterations = 1;
                    data.min_iterations = 1;
                    data.max_iterations = 1;

                    let mut center_reference_zoom = string_to_extended(&data.zoom);
                    center_reference_zoom.exponent += 40;

                    // This is the maximum zoom using the center reference that is valid
                    data.center_reference_zoom = extended_to_string_long(center_reference_zoom);

                    return;
                }

                if command.is(CALCULATE_ROOT) {
                    data.root_stage = 1;

                    let size = ctx.size().to_rect();

                    let top_left = (self.newton_pos1.0.min(self.newton_pos2.0), self.newton_pos1.1.min(self.newton_pos2.1));
                    let bottom_right = (self.newton_pos1.0.max(self.newton_pos2.0), self.newton_pos1.1.max(self.newton_pos2.1));

                    self.root_pos_current = (0.5 * (top_left.0 + bottom_right.0), 0.5 * (top_left.1 + bottom_right.1));
                    self.root_pos_start = self.root_pos_current;
    
                    let i1 = top_left.0 * renderer.image_width as f64 / size.width();
                    let j1 = top_left.1 * renderer.image_height as f64 / size.height();

                    let i2 = bottom_right.0 * renderer.image_width as f64 / size.width();
                    let j2 = bottom_right.1 * renderer.image_height as f64 / size.height();

                    let cos_rotate = renderer.rotate.cos();
                    let sin_rotate = renderer.rotate.sin();

                    self.renderer_zoom = renderer.zoom;
                    self.renderer_rotate = (cos_rotate, sin_rotate);

                    let delta_pixel =  4.0 / ((renderer.image_height - 1) as f64 * renderer.zoom.mantissa);
                    let delta_top_left = get_delta_top_left(delta_pixel, renderer.image_width, renderer.image_height, cos_rotate, sin_rotate);

                    let element1 = ComplexExtended::new(ComplexFixed::new(
                        i1 * delta_pixel * cos_rotate - j1 * delta_pixel * sin_rotate + delta_top_left.re, 
                        i1 * delta_pixel * sin_rotate + j1 * delta_pixel * cos_rotate + delta_top_left.im
                    ), -renderer.zoom.exponent);

                    let element2 = ComplexExtended::new(ComplexFixed::new(
                        i2 * delta_pixel * cos_rotate - j1 * delta_pixel * sin_rotate + delta_top_left.re, 
                        i2 * delta_pixel * sin_rotate + j1 * delta_pixel * cos_rotate + delta_top_left.im
                    ), -renderer.zoom.exponent);

                    let element3 = ComplexExtended::new(ComplexFixed::new(
                        i2 * delta_pixel * cos_rotate - j2 * delta_pixel * sin_rotate + delta_top_left.re, 
                        i2 * delta_pixel * sin_rotate + j2 * delta_pixel * cos_rotate + delta_top_left.im
                    ), -renderer.zoom.exponent);

                    let element4 = ComplexExtended::new(ComplexFixed::new(
                        i1 * delta_pixel * cos_rotate - j2 * delta_pixel * sin_rotate + delta_top_left.re, 
                        i1 * delta_pixel * sin_rotate + j2 * delta_pixel * cos_rotate + delta_top_left.im
                    ), -renderer.zoom.exponent);

                    let box_center = ComplexExtended::new(ComplexFixed::new(
                        0.5 * (i1 + i2) * delta_pixel * cos_rotate - 0.5 * (j1 + j2) * delta_pixel * sin_rotate + delta_top_left.re, 
                        0.5 * (i1 + i2) * delta_pixel * sin_rotate + 0.5 * (j1 + j2) * delta_pixel * cos_rotate + delta_top_left.im
                    ), -renderer.zoom.exponent);

                    renderer.period_finding = BoxPeriod::new(box_center, [element1, element2, element3, element4]);
                    renderer.root_zoom_factor = data.root_zoom_factor;

                    data.sender.lock().send(THREAD_CALCULATE_ROOT).unwrap();

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

                if command.is(RESET_DEFAULT_LOCATION) {
                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name("start.toml")).unwrap();

                    settings.set("real", new_settings.get_str("real").unwrap()).unwrap();
                    settings.set("imag", new_settings.get_str("imag").unwrap()).unwrap();
                    settings.set("zoom", new_settings.get_str("zoom").unwrap()).unwrap();
                    settings.set("iterations", new_settings.get_int("iterations").unwrap()).unwrap();
                    settings.set("rotate", new_settings.get_float("rotate").unwrap()).unwrap();

                    data.real = settings.get_str("real").unwrap();
                    data.imag = settings.get_str("imag").unwrap();
                    data.zoom = settings.get_str("zoom").unwrap().to_uppercase();
                    data.iteration_limit = settings.get_int("iterations").unwrap();
                    data.rotation = settings.get_float("rotate").unwrap();

                    ctx.submit_command(RESET_RENDERER_FULL);
                }

                if let Some(file_info) = command.get(OPEN_FILE) {
                    settings.set("export", "gui").unwrap();

                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    let file_name = file_info.path().file_name().unwrap().to_str().unwrap().split('.').next().unwrap();

                    let mut reset_renderer = false;
                    let mut quick_reset = false;

                    if let Ok(real) = new_settings.get_str("real") {
                        settings.set("real", real.clone()).unwrap();
                        data.real = real;
                        reset_renderer = true;
                    }

                    if let Ok(imag) = new_settings.get_str("imag") {
                        settings.set("imag", imag.clone()).unwrap();
                        data.imag = imag;
                        reset_renderer = true;
                    }

                    if let Ok(zoom) = new_settings.get_str("zoom") {
                        settings.set("zoom", zoom.clone()).unwrap();
                        data.zoom = zoom.to_uppercase();

                        reset_renderer = true;
                    }

                    if let Ok(iterations) = new_settings.get_int("iterations") {
                        settings.set("iterations", iterations).unwrap();
                        data.iteration_limit = iterations;
                        reset_renderer = true;
                    }

                    if let Ok(rotate) = new_settings.get_float("rotate") {
                        settings.set("rotate", rotate).unwrap();
                        data.rotation = rotate;
                        reset_renderer = true;
                    } else {
                        settings.set("rotate", 0.0).unwrap();
                        data.rotation = 0.0;
                    }

                    if let Ok(width) = new_settings.get_int("image_width") {
                        data.image_width = width;
                        settings.set("image_width", width).unwrap();
                        quick_reset = true;
                    }

                    if let Ok(height) = new_settings.get_int("image_height") {
                        data.image_height = height;
                        settings.set("image_height", height).unwrap();
                        quick_reset = true;
                    }

                    if let Ok(order) = new_settings.get_int("approximation_order") {
                        data.order = order;
                        settings.set("approximation_order", order).unwrap();
                        quick_reset = true;
                    }

                    if let Ok(analytic_derivative) = new_settings.get_bool("analytic_derivative") {
                        renderer.data_export.lock().coloring_type = if analytic_derivative {
                            ColoringType::Distance
                        } else {
                            data.coloring_type
                        };

                        settings.set("analytic_derivative", analytic_derivative).unwrap();
                        quick_reset = true;
                    }

                    if let Ok(colour_values) = new_settings.get_array("palette") {
                        // Only reset these if the palette is defined
                        match new_settings.get_float("iteration_division") {
                            Ok(iteration_division) => {
                                settings.set("iteration_division", iteration_division).unwrap();
                                data.iteration_span = iteration_division;
                            }
                            Err(_) => {
                                settings.set("iteration_division", 1.0).unwrap();
                                data.iteration_span = 1.0;
                            }
                        }
    
                        match new_settings.get_float("palette_offset") {
                            Ok(palette_offset) => {
                                settings.set("palette_offset", palette_offset).unwrap();
                                data.iteration_offset = palette_offset;
                            }
                            Err(_) => {
                                settings.set("palette_offset", 0.0).unwrap();
                                data.iteration_offset = 0.0;
                            }
                        }

                        settings.set("palette", colour_values.clone()).unwrap();

                        let palette = colour_values.chunks_exact(3).map(|value| {
                            // We assume the palette is in BGR rather than RGB
                            (value[0].clone().into_int().unwrap() as u8, 
                                value[1].clone().into_int().unwrap() as u8, 
                                value[2].clone().into_int().unwrap() as u8)
                        }).collect::<Vec<(u8, u8, u8)>>();

                        renderer.data_export.lock().change_palette(
                            Some(palette),
                            settings.get_float("iteration_division").unwrap() as f32,
                            settings.get_float("palette_offset").unwrap() as f32
                        );

                        data.palette_source = file_name.to_string();

                        if !reset_renderer || !quick_reset {
                            renderer.data_export.lock().regenerate();
                            ctx.submit_command(REPAINT);
                        }
                    }

                    settings.merge(new_settings).unwrap();

                    if reset_renderer {
                        ctx.submit_command(RESET_RENDERER_FULL);
                    } else if quick_reset {
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

                            let palette = renderer.data_export.lock().palette_buffer.clone().into_iter().flat_map(|seq| {
                                let (r, g, b, _) = seq.rgba_u8();
                                vec![r, g, b]
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
                }
            },
            _ => {}
        }
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FractalData, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &FractalData, _data: &FractalData, _env: &Env) {
        // println!("update called");
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

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, _env: &Env) {
        if self.image_width * self.image_height > 0 {
            if self.needs_buffer_refresh {
                let temporary_image = data.buffer.lock().buffer.clone();

                self.cached_image = Some(ctx
                    .make_image(self.image_width, self.image_height, &temporary_image, ImageFormat::Rgb)
                    .unwrap());

                self.needs_buffer_refresh = false;
            }

            let size = ctx.size().to_rect();

            let interpolation_mode = if self.image_width > size.width() as usize || self.image_height > size.height() as usize {
                InterpolationMode::Bilinear
            } else {
                InterpolationMode::NearestNeighbor
            };

            ctx.draw_image(self.cached_image.as_ref().unwrap(), size, interpolation_mode);

            if self.show_selecting_box {
                let rect = Rect::from_origin_size(self.newton_pos1, (self.newton_pos2.0 - self.newton_pos1.0, self.newton_pos2.1 - self.newton_pos1.1));
                let fill_color = Color::rgba8(0, 0, 0, 150);
                ctx.fill(rect, &fill_color);

                let circle = Circle::new(self.root_pos_current, 2.0);
                let fill_color = Color::rgba8(255, 0, 0, 255);

                ctx.fill(circle, &fill_color);
            }
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

    if settings.get_bool("show_output").unwrap() {
        println!("{:<6}| {:<15}| {:<15}| {:<15}| {:<6}| {:<15}| {:<15}| {:<15}| {:<15}| {:<6}| {:<15}| {:<15}| {:<15}", "Frame", "Zoom", "Approx [ms]", "Skipped [it]", "Order", "Maximum [it]", "Packing [ms]", "Iteration [ms]", "Correct [ms]", "Ref", "Saving [ms]", "Frame [ms]", "TOTAL [ms]");
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
    ).window_size((1386.0, 825.0)).resizable(true).menu(ui::make_menu);

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
            iteration_span: settings.get_float("iteration_division").unwrap(),
            iteration_offset: settings.get_float("palette_offset").unwrap(),
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
            experimental: settings.get_bool("experimental").unwrap(),
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
            center_reference_zoom: extended_to_string_long(center_reference_zoom)
        })
        .expect("launch failed");
}