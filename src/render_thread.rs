use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::{Instant, Duration};
use rust_fractal::renderer::FractalRenderer;
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
                THREAD_RESET_RENDERER_FULL => {
                    let mut renderer = thread_renderer.lock();

                    let thread_counter_1 = renderer.progress.reference.clone();
                    let thread_counter_2 = renderer.progress.series_approximation.clone();
                    let thread_counter_3 = renderer.progress.reference_maximum.clone();
                    let thread_counter_4 = renderer.progress.series_validation.clone();
                    let thread_counter_5 = renderer.progress.iteration.clone();
                    let thread_counter_6 = renderer.progress.glitched_maximum.clone();
                    let thread_counter_7 = renderer.progress.min_series_approximation.clone();
                    let thread_counter_8 = renderer.progress.max_series_approximation.clone();

                    renderer.regenerate_from_settings(thread_settings.lock().clone());

                    let total_pixels = renderer.total_pixels as f64;

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

                                    test.submit_command(UPDATE_PROGRESS, (stage, progress, time, min_valid_iteration, max_valid_iteration), Target::Auto).unwrap();
                                }
                            };

                            if stage > 3 {
                                index += 1;
                                if index % 10 == 0 {
                                    test.submit_command(REPAINT, (), Target::Auto).unwrap();
                                    index = 0;
                                }
                            }
                            
                            // TODO maybe this can be dynamic based on the time to draw the image? e.g. 1ms draw is 60hz, 200ms is 1hz or less
                            thread::sleep(Duration::from_millis(20));
                        };
                    });
                    
                    renderer.render_frame(0, String::from(""), stop_flag);

                    tx.send(()).unwrap();

                    event_sink.submit_command(UPDATE_PROGRESS, (0, 1.0, renderer.render_time as usize, renderer.series_approximation.min_valid_iteration, renderer.series_approximation.max_valid_iteration), Target::Auto).unwrap();
                    event_sink.submit_command(REPAINT, (), Target::Auto).unwrap();
                }
                THREAD_RESET_RENDERER_FAST => {
                    let mut renderer = thread_renderer.lock();

                    let thread_counter_1 = renderer.progress.reference.clone();
                    let thread_counter_2 = renderer.progress.series_approximation.clone();
                    let thread_counter_3 = renderer.progress.reference_maximum.clone();
                    let thread_counter_4 = renderer.progress.series_validation.clone();
                    let thread_counter_5 = renderer.progress.iteration.clone();
                    let thread_counter_6 = renderer.progress.glitched_maximum.clone();
                    let thread_counter_7 = renderer.progress.min_series_approximation.clone();
                    let thread_counter_8 = renderer.progress.max_series_approximation.clone();

                    let total_pixels = renderer.total_pixels as f64;

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

                                    test.submit_command(UPDATE_PROGRESS, (stage, progress, time, min_valid_iteration, max_valid_iteration), Target::Auto).unwrap();
                                }
                            };

                            if stage > 3 {
                                index += 1;

                                if index % 10 == 0 {
                                    test.submit_command(REPAINT, (), Target::Auto).unwrap();
                                    index = 0;
                                }
                            }
        
                            thread::sleep(Duration::from_millis(20));
                        };
                    });

                    renderer.render_frame(1, String::from(""), stop_flag);

                    tx.send(()).unwrap();

                    event_sink.submit_command(UPDATE_PROGRESS, (0, 1.0, renderer.render_time as usize, renderer.series_approximation.min_valid_iteration, renderer.series_approximation.max_valid_iteration), Target::Auto).unwrap();
                    
                    // println!("frames: {}, repeat: {}, zoom: {}", renderer.remaining_frames, repeat_flag.get(), renderer.zoom.to_float());

                    event_sink.submit_command(REPAINT, (), Target::Auto).unwrap();

                    if (renderer.zoom.to_float() > 0.5) && repeat_flag.load(Ordering::SeqCst) {
                        drop(renderer);
                        thread::sleep(Duration::from_millis(100));
                        event_sink.submit_command(MULTIPLY_ZOOM, 0.5, Target::Auto).unwrap();
                    } else {
                        repeat_flag.store(false, Ordering::SeqCst);
                    };
                }
                _ => {
                    println!("thread_command: {}", command);
                }
            }
        }
    }
}