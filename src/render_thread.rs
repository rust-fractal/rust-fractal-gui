use std::sync::mpsc;
use std::time::{Instant, Duration};
use rust_fractal::renderer::FractalRenderer;
use druid::{Selector, Target};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use std::sync::{Arc, Mutex};
use config::Config;
use std::thread;


pub fn testing_renderer(
    event_sink: druid::ExtEventSink, 
    reciever: mpsc::Receiver<String>, 
    thread_settings: Arc<Mutex<Config>>, 
    thread_renderer: Arc<Mutex<FractalRenderer>>, 
    thread_stop_flag: Arc<RelaxedCounter>,
    thread_repeat_flag: Arc<RelaxedCounter>) {
    loop {
        let stop_flag = thread_stop_flag.clone();
        let repeat_flag = thread_repeat_flag.clone();

        match reciever.recv() {
            Ok(command) => {
                // execute commands
                match command.as_ref() {
                    "reset_renderer_full" => {
                        let mut renderer = thread_renderer.lock().unwrap();

                        *renderer = FractalRenderer::new(thread_settings.lock().unwrap().clone());

                        let total_pixels = (renderer.image_width * renderer.image_height) as f64;

                        let (tx, rx) = mpsc::channel();

                        let test = event_sink.clone();

                        let thread_counter_1 = renderer.progress.reference.clone();
                        let thread_counter_2 = renderer.progress.series_approximation.clone();
                        let thread_counter_3 = renderer.progress.reference_maximum.clone();
                        let thread_counter_4 = renderer.progress.series_validation.clone();
                        let thread_counter_5 = renderer.progress.iteration.clone();
                        let thread_counter_6 = renderer.progress.glitched_maximum.clone();
                        let thread_counter_7 = renderer.progress.min_series_approximation.clone();
                        let thread_counter_8 = renderer.progress.max_series_approximation.clone();

                        thread::spawn(move || {
                            let start = Instant::now();

                            loop {
                                match rx.try_recv() {
                                    Ok(_) => {
                                        break;
                                    },
                                    Err(_) => {
                                        let series_validation_progress = thread_counter_4.get();

                                        let mut progress = 0.0;
                                        let mut stage = 1usize;

                                        // Less than two means that the series validation has not completed
                                        if series_validation_progress < 2 {
                                            let series_approximation_amount = thread_counter_2.get();

                                            let reference_progress = thread_counter_1.get() as f64;
                                            let series_approximation_progress = series_approximation_amount as f64;
                                            let reference_maximum = thread_counter_3.get() as f64;

                                            if series_approximation_amount == 0 {
                                                progress = reference_progress / reference_maximum
                                            } else {
                                                stage = 2;

                                                progress += 0.9 * series_approximation_progress / reference_maximum;
                                                progress += 0.1 * series_validation_progress as f64 / 2.0;
                                            }
                                        } else {
                                            let glitched_amount = thread_counter_6.get();

                                            if glitched_amount != 0 {
                                                let complete_amount = total_pixels as f64 - glitched_amount as f64;

                                                stage = 4;
                                                progress = (thread_counter_5.get() as f64 - complete_amount) / glitched_amount as f64
                                            } else {
                                                stage = 3;
                                                progress = thread_counter_5.get() as f64 / total_pixels
                                            }
                                        };

                                        let time = start.elapsed().as_millis() as usize;
                                        let min_valid_iteration = thread_counter_7.get();
                                        let max_valid_iteration = thread_counter_8.get();
            
                                        test.submit_command(
                                            Selector::new("update_progress"), (stage, progress, time, min_valid_iteration, max_valid_iteration), Target::Auto).unwrap();
                                    }
                                };
            
                                thread::sleep(Duration::from_millis(10));
                            };
                        });
                        
                        renderer.render_frame(0, String::from(""), Some(stop_flag));

                        tx.send(()).unwrap();

                        event_sink.submit_command(
                            Selector::new("update_progress"), 
                            (0usize, 1.0, renderer.render_time as usize, renderer.series_approximation.min_valid_iteration, renderer.series_approximation.max_valid_iteration), 
                            Target::Auto).unwrap();

                        event_sink.submit_command(
                            Selector::new("repaint"), (), Target::Auto).unwrap();
                    }
                    "reset_renderer_fast" => {
                        let mut renderer = thread_renderer.lock().unwrap();

                        let total_pixels = (renderer.image_width * renderer.image_height) as f64;

                        let (tx, rx) = mpsc::channel();

                        let test = event_sink.clone();

                        let thread_counter_1 = renderer.progress.reference.clone();
                        let thread_counter_2 = renderer.progress.series_approximation.clone();
                        let thread_counter_3 = renderer.progress.reference_maximum.clone();
                        let thread_counter_4 = renderer.progress.series_validation.clone();
                        let thread_counter_5 = renderer.progress.iteration.clone();
                        let thread_counter_6 = renderer.progress.glitched_maximum.clone();
                        let thread_counter_7 = renderer.progress.min_series_approximation.clone();
                        let thread_counter_8 = renderer.progress.max_series_approximation.clone();

                        thread::spawn(move || {
                            let start = Instant::now();

                            loop {
                                match rx.try_recv() {
                                    Ok(_) => {
                                        break;
                                    },
                                    Err(_) => {
                                        let series_validation_progress = thread_counter_4.get();

                                        let mut progress = 0.0;
                                        let mut stage = 1usize;

                                        // Less than two means that the series validation has not completed
                                        if series_validation_progress < 2 {
                                            let series_approximation_amount = thread_counter_2.get();

                                            let reference_progress = thread_counter_1.get() as f64;
                                            let series_approximation_progress = series_approximation_amount as f64;
                                            let reference_maximum = thread_counter_3.get() as f64;

                                            if series_approximation_amount == 0 {
                                                progress = reference_progress / reference_maximum
                                            } else {
                                                stage = 2;

                                                progress += 0.9 * series_approximation_progress / reference_maximum;
                                                progress += 0.1 * series_validation_progress as f64 / 2.0;
                                            }
                                        } else {
                                            let glitched_amount = thread_counter_6.get();

                                            if glitched_amount != 0 {
                                                let complete_amount = total_pixels as f64 - glitched_amount as f64;

                                                stage = 4;
                                                progress = (thread_counter_5.get() as f64 - complete_amount) / glitched_amount as f64
                                            } else {
                                                stage = 3;
                                                progress = thread_counter_5.get() as f64 / total_pixels
                                            }
                                        };

                                        let time = start.elapsed().as_millis() as usize;
                                        let min_valid_iteration = thread_counter_7.get();
                                        let max_valid_iteration = thread_counter_8.get();

                                        test.submit_command(
                                            Selector::new("update_progress"), (stage, progress, time, min_valid_iteration, max_valid_iteration), Target::Auto).unwrap();
                                    }
                                };
            
                                thread::sleep(Duration::from_millis(10));
                            };
                        });

                        renderer.render_frame(1, String::from(""), Some(stop_flag));

                        tx.send(()).unwrap();

                        event_sink.submit_command(
                            Selector::new("update_progress"), 
                            (0usize, 1.0, renderer.render_time as usize, renderer.series_approximation.min_valid_iteration, renderer.series_approximation.max_valid_iteration), 
                            Target::Auto).unwrap();

                        println!("frames: {}, repeat: {}, zoom: {}", renderer.remaining_frames, repeat_flag.get(), renderer.zoom.to_float());

                        event_sink.submit_command(
                            Selector::new("repaint"), (), Target::Auto).unwrap();

                        if (renderer.zoom.to_float() > 0.5) && repeat_flag.get() == 0 {
                            drop(renderer);

                            thread::sleep(Duration::from_millis(100));

                            println!("sending multiply command");
                            event_sink.submit_command(Selector::new("multiply_zoom_level"), 0.5, Target::Auto).unwrap();
                        } else {
                            println!("not repeating any more");
                            repeat_flag.inc();
                        };

                        
                    }
                    _ => {
                        println!("thread_command: {}", command);
                    }
                }
            }
            _ => {}
        }
    }
}