use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Instant, Duration};

use rust_fractal::{renderer::FractalRenderer, util::ComplexExtended};
use rust_fractal::util::{FloatArbitrary, linear_interpolation_between_zoom, extended_to_string_long};
use rust_fractal::math::{get_nucleus, get_nucleus_position};

use druid::Target;
use std::sync::Arc;
use parking_lot::Mutex;
use config::Config;
use std::thread;

use crate::commands::*;


pub fn testing_renderer(
    event_sink: druid::ExtEventSink, 
    reciever: mpsc::Receiver<usize>, 
    thread_settings: Arc<Mutex<Config>>, 
    thread_renderer: Arc<Mutex<FractalRenderer>>, 
    thread_stop_flag: Arc<AtomicBool>,
    thread_repeat_flag: Arc<AtomicBool>) {
    loop {
        let stop_flag = thread_stop_flag.clone();
        let repeat_flag = thread_repeat_flag.clone();

        if let Ok(command) = reciever.recv() {
            match command {
                THREAD_RESET_RENDERER_FAST | THREAD_RESET_RENDERER_FULL => {
                    let mut renderer = thread_renderer.lock();

                    let thread_counter_1 = renderer.progress.reference.clone();
                    let thread_counter_2 = renderer.progress.series_approximation.clone();
                    let thread_counter_3 = renderer.progress.reference_maximum.clone();
                    let thread_counter_4 = renderer.progress.series_validation.clone();
                    let thread_counter_5 = renderer.progress.iteration.clone();
                    let thread_counter_6 = renderer.progress.glitched_maximum.clone();
                    let thread_counter_7 = renderer.progress.min_series_approximation.clone();
                    let thread_counter_8 = renderer.progress.max_series_approximation.clone();
                    let thread_counter_9 = renderer.progress.reference_count.clone();

                    if command == THREAD_RESET_RENDERER_FULL {
                        renderer.regenerate_from_settings(thread_settings.lock().clone());
                    }

                    let total_pixels = renderer.total_pixels as f64;

                    let repaint_frequency = (renderer.total_pixels / 200000).max(1);

                    let (tx, rx) = mpsc::channel();

                    let test = event_sink.clone();

                    thread::spawn(move || {
                        let start = Instant::now();

                        let mut index = 0;
                        let mut stage = 1usize;

                        loop {
                            match rx.try_recv() {
                                Ok(_) => {
                                    break;
                                },
                                Err(_) => {
                                    let series_validation_progress = thread_counter_4.load(Ordering::Relaxed);

                                    let mut progress = 0.0;
                                    
                                    // Less than two means that the series validation has not completed
                                    if series_validation_progress < 2 {
                                        let series_approximation_amount = thread_counter_2.load(Ordering::Relaxed);

                                        let reference_progress = thread_counter_1.load(Ordering::Relaxed) as f64;
                                        let series_approximation_progress = series_approximation_amount as f64;
                                        let reference_maximum = thread_counter_3.load(Ordering::Relaxed) as f64;

                                        if series_approximation_amount == 0 {
                                            progress = reference_progress / reference_maximum
                                        } else {
                                            stage = if series_approximation_progress / reference_maximum >= 1.0 {
                                                3
                                            } else {
                                                2
                                            };

                                            progress += 0.9 * series_approximation_progress / reference_maximum;
                                            progress += 0.1 * series_validation_progress as f64 / 2.0;
                                        }
                                    } else {
                                        let glitched_amount = thread_counter_6.load(Ordering::Relaxed);

                                        if glitched_amount != 0 {
                                            let complete_amount = total_pixels as f64 - glitched_amount as f64;

                                            stage = 5;
                                            progress = (thread_counter_5.load(Ordering::Relaxed) as f64 - complete_amount) / glitched_amount as f64
                                        } else {
                                            stage = 4;
                                            progress = thread_counter_5.load(Ordering::Relaxed) as f64 / total_pixels
                                        }
                                    };

                                    let time = start.elapsed().as_millis() as usize;
                                    let min_valid_iteration = thread_counter_7.load(Ordering::Relaxed);
                                    let max_valid_iteration = thread_counter_8.load(Ordering::Relaxed);
                                    let reference_count = thread_counter_9.load(Ordering::Relaxed);

                                    test.submit_command(UPDATE_RENDERING_PROGRESS, (stage, progress, time, min_valid_iteration, max_valid_iteration, reference_count), Target::Auto).unwrap();
                                }
                            };

                            if stage > 3 {
                                index += 1;
                                if index % repaint_frequency == 0 {
                                    test.submit_command(REPAINT, (), Target::Auto).unwrap();
                                    index = 0;
                                }
                            }
                            
                            thread::sleep(Duration::from_millis(20));
                        };
                    });
                    
                    if command == THREAD_RESET_RENDERER_FULL {
                        renderer.render_frame(0, String::from(""), stop_flag);
                    } else {
                        renderer.render_frame(1, String::from(""), stop_flag);
                    }

                    tx.send(()).unwrap();

                    event_sink.submit_command(UPDATE_RENDERING_PROGRESS, (0, 1.0, renderer.render_time as usize, renderer.series_approximation.min_valid_iteration, renderer.series_approximation.max_valid_iteration, renderer.progress.reference_count.load(Ordering::SeqCst)), Target::Auto).unwrap();
                    event_sink.submit_command(REPAINT, (), Target::Auto).unwrap();

                    if command == THREAD_RESET_RENDERER_FAST {
                        if (renderer.zoom.to_float() > 0.5) && repeat_flag.load(Ordering::SeqCst) {
                            let zoom_out_factor = 1.0 / renderer.zoom_scale_factor;
                            drop(renderer);

                            // This is the delay between frames of zoom animations
                            thread::sleep(Duration::from_millis(100));

                            event_sink.submit_command(MULTIPLY_ZOOM, zoom_out_factor, Target::Auto).unwrap();
                        } else {
                            repeat_flag.store(false, Ordering::SeqCst);
                        };
                    }
                }
                THREAD_CALCULATE_ROOT => {
                    let stop_flag = thread_stop_flag.clone();

                    let mut renderer = thread_renderer.lock();

                    renderer.find_period();

                    event_sink.submit_command(SET_PERIOD, renderer.period_finding.period, Target::Auto).unwrap();
                    
                    let mut box_center_arbitrary = renderer.center_reference.c.clone();
                    let box_center = renderer.period_finding.box_center;

                    let temp = FloatArbitrary::with_val(renderer.center_reference.c.real().prec(), box_center.exponent).exp2();
            
                    *box_center_arbitrary.mut_real() += temp.clone() * box_center.mantissa.re;
                    *box_center_arbitrary.mut_imag() += temp.clone() * box_center.mantissa.im;

                    let thread_counter_1 = Arc::new(AtomicUsize::new(0));
                    let thread_counter_1_clone = thread_counter_1.clone();

                    let thread_counter_2 = Arc::new(AtomicUsize::new(0));
                    let thread_counter_2_clone = thread_counter_2.clone();

                    let current_estimate_difference_1 = Arc::new(Mutex::new(ComplexExtended::new2(0.0, 0.0, -99999999)));
                    let current_estimate_difference_2 = current_estimate_difference_1.clone();

                    let (tx, rx) = mpsc::channel();

                    let test = event_sink.clone();

                    thread::spawn(move || {
                        loop {
                            match rx.try_recv() {
                                Ok(_) => {
                                    break;
                                },
                                Err(_) => {
                                    // do some processing to get back to the original coordinates
                                    test.submit_command(UPDATE_ROOT_PROGRESS, (thread_counter_1.load(Ordering::Relaxed), thread_counter_2.load(Ordering::Relaxed), *current_estimate_difference_1.lock()), Target::Auto).unwrap();
                                }
                            }
                            
                            thread::sleep(Duration::from_millis(20));
                        };
                    });
                    
                    if let Some(nucleus) = get_nucleus(box_center_arbitrary, renderer.period_finding.period, thread_counter_1_clone, thread_counter_2_clone, stop_flag, current_estimate_difference_2) {
                        let nucleus_position = get_nucleus_position(nucleus.clone(), renderer.period_finding.period);
                    
                        let new_zoom = linear_interpolation_between_zoom(renderer.zoom, nucleus_position.0, renderer.root_zoom_factor);
    
                        drop(renderer);
    
                        let mut settings = thread_settings.lock();
    
                        settings.set("real", nucleus.real().to_string()).unwrap();
                        settings.set("imag", nucleus.imag().to_string()).unwrap();
                        settings.set("zoom", extended_to_string_long(new_zoom)).unwrap();
    
                        drop(settings);
    
                        event_sink.submit_command(ROOT_FINDING_COMPLETE, Some(nucleus_position.0), Target::Auto).unwrap();
    
                        // this currently updates the data fields
                        event_sink.submit_command(REVERT_LOCATION, (), Target::Auto).unwrap();
                        event_sink.submit_command(RESET_RENDERER_FULL, (), Target::Auto).unwrap();
                    } else {
                        event_sink.submit_command(ROOT_FINDING_COMPLETE, None, Target::Auto).unwrap();
                    }

                    tx.send(()).unwrap();
                }
                _ => {
                    println!("thread_command: {}", command);
                }
            }
        }
    }
}