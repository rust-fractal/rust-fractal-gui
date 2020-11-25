use std::sync::{Arc, Mutex};

use druid::widget::prelude::*;

use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command, Data, Lens, Selector};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::theme::{BUTTON_BORDER_RADIUS, TEXT_SIZE_NORMAL, FONT_NAME, TEXTBOX_BORDER_RADIUS, PROGRESS_BAR_RADIUS};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long, string_to_extended};

use config::{Config, File};

// use std::thread;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use atomic_counter::AtomicCounter;

mod ui;
pub mod lens;

struct FractalWidget {}

#[derive(Clone, Data, Lens)]
pub struct FractalData {
    updated: usize,
    temporary_width: i64,
    temporary_height: i64,
    temporary_real: String,
    temporary_imag: String,
    temporary_zoom: String,
    temporary_iterations: i64,
    temporary_rotation: f64,
    temporary_order: i64,
    temporary_palette_source: String,
    temporary_iteration_division: String,
    temporary_iteration_offset: String,
    temporary_progress1: f64,
    temporary_progress2: f64,
    temporary_progress3: f64,
    renderer: Arc<Mutex<FractalRenderer>>,
    settings: Arc<Mutex<Config>>,
    sender: Arc<Mutex<mpsc::Sender<String>>>
}

impl Widget<FractalData> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        ctx.request_focus();

        match event {
            Event::WindowConnected => {
                let settings = data.settings.lock().unwrap();

                data.temporary_width = settings.get_int("image_width").unwrap();
                data.temporary_height = settings.get_int("image_height").unwrap();

                let sender = data.sender.lock().unwrap();
                sender.send(String::from("reset_renderer_full")).unwrap();

                data.updated += 1;
            }
            Event::MouseDown(e) => {
                let mut settings = data.settings.lock().unwrap();
                let mut renderer = data.renderer.lock().unwrap();

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

                        // Set the overrides for the current location
                        settings.set("real", location.real().to_string()).unwrap();
                        settings.set("imag", location.imag().to_string()).unwrap();
                        settings.set("zoom", extended_to_string_long(zoom)).unwrap();

                        data.temporary_real = settings.get_str("real").unwrap();
                        data.temporary_imag = settings.get_str("imag").unwrap();
                        data.temporary_zoom = settings.get_str("zoom").unwrap();

                        // data.derive_from_settings(&self.current_settings, self.renderer.as_ref().unwrap());
                        renderer.maximum_iteration = settings.get_int("iterations").unwrap() as usize;
                        renderer.update_location(zoom, location);

                        // BUG, somewhere in this update thing, need to deal with if the maximum iteration is less than reference or something
                        settings.set("iterations", renderer.maximum_iteration as i64).unwrap();
                        data.temporary_iterations = renderer.maximum_iteration as i64;

                        ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
                    } else {
                        ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 0.5), None);
                    }
                }
            },
            Event::KeyUp(e) => {
                // Shortcut keys
                if e.key_code == KeyCode::KeyD {
                    ctx.submit_command(Command::new(Selector::new("toggle_derivative"), ()), None);
                }

                if e.key_code == KeyCode::KeyZ {
                    ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 2.0), None);
                }

                if e.key_code == KeyCode::KeyO {
                    ctx.submit_command(Command::new(
                        Selector::new("open_location"), 
                        ()
                    ), None);
                }

                if e.key_code == KeyCode::KeyN {
                    ctx.submit_command(Command::new(Selector::new("native_image_size"), ()), None);
                }

                if e.key_code == KeyCode::KeyT {
                    ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 0.5), None);
                }

                if e.key_code == KeyCode::KeyY {
                    ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 2.0), None);
                }

                if e.key_code == KeyCode::KeyR {
                    let settings = data.settings.lock().unwrap();
                    let new_rotate = (settings.get_float("rotate").unwrap() + 15.0) % 360.0;

                    ctx.submit_command(Command::new(Selector::new("set_rotation"), new_rotate), None);
                }
            },
            Event::Command(command) => {
                // println!("{:?}", command);

                if let Some(_) = command.get::<()>(Selector::new("repaint")) {
                    data.updated += 1;
                    ctx.request_paint();
                    return;
                }

                if let Some((progress1, progress2, progress3)) = command.get::<(f64, f64, f64)>(Selector::new("update_progress")) {
                    data.temporary_progress1 = *progress1;
                    data.temporary_progress2 = *progress2;
                    data.temporary_progress3 = *progress3;

                    return;
                }


                let mut settings = data.settings.lock().unwrap();
                let mut renderer = data.renderer.lock().unwrap();

                if let Some(factor) = command.get::<f64>(Selector::new("multiply_image_size")) {
                    let new_width = settings.get_int("image_width").unwrap() as f64 * factor;
                    let new_height = settings.get_int("image_height").unwrap() as f64 * factor;

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (new_width as i64, new_height as i64)), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("native_image_size")) {
                    let window_width = settings.get_float("window_width").unwrap();
                    let window_height = settings.get_float("window_height").unwrap();

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (window_width as i64, window_height as i64)), None);
                    return;
                }

                if let Some(dimensions) = command.get::<(i64, i64)>(Selector::new("set_image_size")) {
                    if dimensions.0 as usize == renderer.image_width && dimensions.1 as usize == renderer.image_height {
                        return;
                    }

                    settings.set("image_width", dimensions.0 as i64).unwrap();
                    settings.set("image_height", dimensions.1 as i64).unwrap();

                    renderer.image_width = dimensions.0 as usize;
                    renderer.image_height = dimensions.1 as usize;

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(iterations) = command.get::<i64>(Selector::new("set_iterations")) {
                    if *iterations as usize == renderer.data_export.maximum_iteration {
                        return;
                    }

                    settings.set("iterations", *iterations).unwrap();
                    data.temporary_iterations = *iterations;

                    if *iterations as usize <= renderer.maximum_iteration {
                        renderer.data_export.maximum_iteration = data.temporary_iterations as usize;
                        renderer.data_export.regenerate();

                        data.updated += 1;
                        ctx.request_paint();
                        return;
                    }

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("set_approximation_order")) {
                    if (data.temporary_order as usize) == renderer.series_approximation.order {
                        return;
                    }

                    if (data.temporary_order as usize) > 128 {
                        data.temporary_order = 128;
                    }

                    if (data.temporary_order as usize) < 4 {
                        data.temporary_order = 4;
                    }

                    settings.set("approximation_order", data.temporary_order).unwrap();
                    renderer.series_approximation.order = data.temporary_order as usize;
                    renderer.progress.reset_series_approximation();

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("set_location")) {
                    let current_real = settings.get_str("real").unwrap();
                    let current_imag = settings.get_str("imag").unwrap();
                    let current_zoom = settings.get_str("zoom").unwrap();
                    let current_iterations = settings.get_int("iterations").unwrap();
                    let current_rotation = settings.get_float("rotate").unwrap();

                    if current_real == data.temporary_real && current_imag == data.temporary_imag {
                        // Check if the zoom has decreased or is near to the current level
                        if current_zoom.to_uppercase() == data.temporary_zoom.to_uppercase() {
                            // nothing has changed
                            if current_rotation == data.temporary_rotation && current_iterations == data.temporary_iterations {
                                println!("nothing");
                                return;
                            }

                            // iterations changed
                            if current_iterations == data.temporary_iterations {
                                println!("rotation");
                                ctx.submit_command(Command::new(Selector::new("set_rotation"), data.temporary_rotation), None);
                                return;
                            }

                            if current_rotation == data.temporary_rotation {
                                println!("iterations");
                                ctx.submit_command(Command::new(Selector::new("set_iterations"), data.temporary_iterations), None);
                                return;
                            }

                            println!("rotation & iterations");

                            settings.set("iterations", data.temporary_iterations).unwrap();

                            if (data.temporary_iterations as usize) < renderer.maximum_iteration {
                                // TODO needs to make it so that pixels are only iterated to the right level
                                renderer.maximum_iteration = data.temporary_iterations as usize;
                                ctx.submit_command(Command::new(Selector::new("set_rotation"), data.temporary_rotation), None);
                                return;
                            }
                        } else {
                            // Zoom has changed, and need to rerender depending on if the zoom has changed too much

                            let current_exponent = renderer.center_reference.zoom.exponent;
                            let new_zoom = string_to_extended(&data.temporary_zoom.to_uppercase());

                            if new_zoom.exponent <= current_exponent {
                                println!("zoom decreased");
                                renderer.zoom = new_zoom;
                                settings.set("zoom", data.temporary_zoom.clone()).unwrap();

                                ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                                return;
                            }
                        }
                    }

                    println!("location changed / zoom increased / iterations increased and rotation");

                    settings.set("real", data.temporary_real.clone()).unwrap();
                    settings.set("imag", data.temporary_imag.clone()).unwrap();
                    settings.set("zoom", data.temporary_zoom.clone()).unwrap();
                    settings.set("rotate", data.temporary_rotation.clone()).unwrap();
                    settings.set("iterations", data.temporary_iterations.clone()).unwrap();

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
                    return;
                }

                if let Some(factor) = command.get::<f64>(Selector::new("multiply_zoom_level")) {
                    renderer.zoom.mantissa *= factor;
                    renderer.zoom.reduce();

                    settings.set("zoom", extended_to_string_long(renderer.zoom)).unwrap();
                    data.temporary_zoom = settings.get_str("zoom").unwrap();

                    // TODO properly set the maximum iterations
                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("toggle_derivative")) {
                    let current_derivative = renderer.data_export.analytic_derivative;
                    settings.set("analytic_derivative", !current_derivative).unwrap();
                    renderer.data_export.analytic_derivative = !current_derivative;

                    // We have already computed the iterations and analytic derivatives
                    if renderer.analytic_derivative {
                        renderer.data_export.regenerate();
                        data.updated += 1;
                    } else {
                        ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    }

                    return;
                }

                if let Some(rotation) = command.get::<f64>(Selector::new("set_rotation")) {
                    let new_rotate = (rotation % 360.0 + 360.0) % 360.0;

                    settings.set("rotate", new_rotate).unwrap();
                    data.temporary_rotation = new_rotate;

                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    renderer.rotate = new_rotate.to_radians();

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("set_offset_division")) {
                    let current_division = settings.get_float("iteration_division").unwrap() as f32;
                    let current_offset = settings.get_float("palette_offset").unwrap() as f32;

                    let new_division = data.temporary_iteration_division.parse::<f32>().unwrap();
                    let new_offset = data.temporary_iteration_offset.parse::<f32>().unwrap() % renderer.data_export.palette.len() as f32;

                    // println!("{} {} {}", data.temporary_iteration_offset, new_offset, new_division);

                    if current_division == new_division && current_offset == new_offset {
                        return;
                    }

                    data.temporary_iteration_division = new_division.to_string();
                    data.temporary_iteration_offset = new_offset.to_string();

                    settings.set("iteration_division", new_division as f64).unwrap();
                    settings.set("palette_offset", new_offset as f64).unwrap();

                    renderer.data_export.iteration_division = new_division;
                    renderer.data_export.iteration_offset = new_offset;

                    renderer.data_export.regenerate();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("reset_renderer_fast")) {
                    renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                    let sender = data.sender.lock().unwrap();
                    sender.send(String::from("reset_renderer_fast")).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("reset_renderer_full")) {
                    let sender = data.sender.lock().unwrap();
                    sender.send(String::from("reset_renderer_full")).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.temporary_order = settings.get_int("approximation_order").unwrap();
                    data.updated += 1;

                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("open_location")) {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let open_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_OPEN_PANEL,
                        open_dialog_options.clone(),
                    ), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("save_location")) {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let save_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_SAVE_PANEL,
                        save_dialog_options.clone(),
                    ), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("save_image")) {
                    let png = FileSpec::new("Portable Network Graphics", &["png"]);
                    let jpg = FileSpec::new("JPEG", &["jpg"]);

                    let save_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![png, jpg]);

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_SAVE_PANEL,
                        save_dialog_options.clone(),
                    ), None);
                    return;
                }

                if let Some(file_info) = command.get(commands::OPEN_FILE) {
                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    let mut reset_renderer = false;

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
                            data.temporary_zoom = zoom;
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

                    match new_settings.get_array("palette") {
                        Ok(colour_values) => {
                            // Only reset these if the palette is defined
                            match new_settings.get_float("iteration_division") {
                                Ok(iteration_division) => {
                                    settings.set("iteration_division", iteration_division).unwrap();
                                    data.temporary_iteration_division = iteration_division.to_string();
                                }
                                Err(_) => {
                                    settings.set("iteration_division", "1").unwrap();
                                    data.temporary_iteration_division = String::from("1");
                                }
                            }
        
                            match new_settings.get_float("palette_offset") {
                                Ok(palette_offset) => {
                                    settings.set("palette_offset", palette_offset).unwrap();
                                    data.temporary_iteration_offset = palette_offset.to_string();
                                }
                                Err(_) => {
                                    settings.set("palette_offset", "0").unwrap();
                                    data.temporary_iteration_offset = String::from("0");
                                }
                            }

                            settings.set("palette", colour_values.clone()).unwrap();

                            let palette = colour_values.chunks_exact(3).map(|value| {
                                // We assume the palette is in BGR rather than RGB
                                (value[2].clone().into_int().unwrap() as u8, 
                                    value[1].clone().into_int().unwrap() as u8, 
                                    value[0].clone().into_int().unwrap() as u8)
                            }).collect::<Vec<(u8, u8, u8)>>();

                            renderer.data_export.palette = palette;
                            renderer.data_export.iteration_division = settings.get_float("iteration_division").unwrap() as f32;
                            renderer.data_export.iteration_offset = settings.get_float("palette_offset").unwrap() as f32;

                            data.temporary_palette_source = file_info.path().file_name().unwrap().to_str().unwrap().to_string();

                            if !reset_renderer {
                                renderer.data_export.regenerate();
                                data.updated += 1;
                                ctx.request_paint();
                            }
                        }
                        Err(_) => {}
                    }

                    settings.merge(new_settings).unwrap();

                    if reset_renderer {
                        ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
                    }

                    return;
                }

                if let Some(file_info) = command.get(commands::SAVE_FILE) {
                    match file_info.clone().unwrap().path().extension().unwrap().to_str().unwrap() {
                        "png" | "jpg" => {
                            renderer.data_export.save_colour(file_info.clone().unwrap().path().to_str().unwrap());
                        },
                        _ => {
                            let real = settings.get_str("real").unwrap();
                            let imag = settings.get_str("imag").unwrap();
                            let zoom = settings.get_str("zoom").unwrap();
                            let iterations = settings.get_int("iterations").unwrap();
                            let rotate = settings.get_float("rotate").unwrap();

                            let output = format!("real = \"{}\"\nimag = \"{}\"\nzoom = \"{}\"\niterations = {}\nrotate = {}", real, imag, zoom, iterations.to_string(), rotate.to_string());

                            if let Err(e) = std::fs::write(file_info.clone().unwrap().path(), output) {
                                println!("Error writing file: {}", e);
                            }
                        }
                    }

                    return;
                }
            },
            _ => {}
        }
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &FractalData, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &FractalData, _data: &FractalData, _env: &Env) {}

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FractalData, _env: &Env) -> Size {
        let mut test = bc.max();

        let mut settings = data.settings.lock().unwrap();

        settings.set("window_width", test.width).unwrap();
        settings.set("window_height", test.height).unwrap();

        test.height = test.width * settings.get_int("image_height").unwrap() as f64 / settings.get_int("image_width").unwrap() as f64;

        test
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, _env: &Env) {
        let size = ctx.size().to_rect();
        let renderer = data.renderer.lock().unwrap();

        let image = ctx
            .make_image(renderer.image_width, renderer.image_height, &renderer.data_export.rgb, ImageFormat::Rgb)
            .unwrap();

        if renderer.image_width > size.width() as usize {
            ctx.draw_image(&image, size, InterpolationMode::Bilinear);
        } else {
            ctx.draw_image(&image, size, InterpolationMode::NearestNeighbor);
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

    let window = WindowDesc::new(ui::ui_builder).title(
        LocalizedString::new("rust-fractal-gui"),
    ).window_size((1280.0, 720.0)).resizable(true);

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, reciever) = mpsc::channel();
    let (sender2, reciever2) = mpsc::channel();

    let shared_settings = Arc::new(Mutex::new(settings.clone()));

    let testing = shared_settings.clone();

    thread::spawn(move || testing_renderer(event_sink, reciever, testing, sender2));

    let arc_stuff = reciever2.recv().unwrap();

    launcher
        // .use_simple_logger()
        .configure_env(|env, _| {
            env.set(FONT_NAME, "Lucida Console");
            env.set(TEXT_SIZE_NORMAL, 12.0);
            env.set(BUTTON_BORDER_RADIUS, 2.0);
            env.set(TEXTBOX_BORDER_RADIUS, 2.0);
            env.set(PROGRESS_BAR_RADIUS, 2.0);
            // for test in env.get_all() {
            //     println!("{:?}", test);
            // };
        })
        .launch(FractalData {
            updated: 0,
            temporary_width: settings.get_int("image_width").unwrap(),
            temporary_height: settings.get_int("image_height").unwrap(),
            temporary_real: settings.get_str("real").unwrap(),
            temporary_imag: settings.get_str("imag").unwrap(),
            temporary_zoom: settings.get_str("zoom").unwrap(),
            temporary_iterations: settings.get_int("iterations").unwrap(),
            temporary_rotation: settings.get_float("rotate").unwrap(),
            temporary_order: settings.get_int("approximation_order").unwrap(),
            temporary_palette_source: "default".to_string(),
            temporary_iteration_division: settings.get_float("iteration_division").unwrap().to_string(),
            temporary_iteration_offset: settings.get_float("palette_offset").unwrap().to_string(),
            temporary_progress1: 0.0,
            temporary_progress2: 0.0,
            temporary_progress3: 0.0,
            renderer: arc_stuff,
            settings: shared_settings,
            sender: Arc::new(Mutex::new(sender)),
        })
        .expect("launch failed");
}

fn testing_renderer(event_sink: druid::ExtEventSink, reciever: mpsc::Receiver<String>, settings: Arc<Mutex<Config>>, sender2: mpsc::Sender<Arc<Mutex<FractalRenderer>>>) {
    let renderer_reference = Arc::new(Mutex::new(FractalRenderer::new(settings.lock().unwrap().clone())));

    sender2.send(renderer_reference.clone()).unwrap();

    loop {
        match reciever.try_recv() {
            Ok(command) => {
                // execute commands
                match command.as_ref() {
                    "reset_renderer_full" => {
                        let mut renderer = renderer_reference.lock().unwrap();

                        *renderer = FractalRenderer::new(settings.lock().unwrap().clone());

                        let total_pixels = (renderer.image_width * renderer.image_height) as f64;

                        let (tx, rx) = mpsc::channel();

                        let test = event_sink.clone();

                        let thread_counter_1 = renderer.progress.reference.clone();
                        let thread_counter_2 = renderer.progress.series_approximation.clone();
                        let thread_counter_3 = renderer.progress.reference_maximum.clone();
                        let thread_counter_4 = renderer.progress.series_validation.clone();
                        let thread_counter_5 = renderer.progress.iteration.clone();
                        let thread_counter_6 = renderer.progress.glitched_maximum.clone();

                        thread::spawn(move || {
                            loop {
                                match rx.try_recv() {
                                    Ok(_) => {
                                        break;
                                    },
                                    Err(_) => {
                                        // 40% weighting to first reference, 40% to SA calculation, 20% to SA checking
                                        let mut progress1 = 0.0;

                                        progress1 += 0.45 * thread_counter_1.get() as f64 / thread_counter_3.get() as f64;
                                        progress1 += 0.45 * thread_counter_2.get() as f64 / thread_counter_3.get() as f64;
                                        progress1 += 0.1 * thread_counter_4.get() as f64 / 2.0;

                                        // println!("reference: {}, series_approximation: {}, reference_maximum: {}", thread_counter_1.get(), thread_counter_2.get(), thread_counter_3.get());

                                        let glitched_amount = thread_counter_6.get();
                                        let complete_amount = total_pixels as f64 - glitched_amount as f64;

                                        let (progress2, progress3) = if glitched_amount != 0 {
                                            (1.0, (thread_counter_5.get() as f64 - complete_amount) / glitched_amount as f64)
                                        } else {
                                            (thread_counter_5.get() as f64 / total_pixels, 0.0)
                                        };
            
                                        test.submit_command(
                                            Selector::new("update_progress"), (progress1, progress2, progress3), None).unwrap();
                                    }
                                };
            
                                thread::sleep(Duration::from_millis(50));
                            };
                        });
                        
                        renderer.render_frame(0, String::from(""));

                        tx.send(()).unwrap();

                        event_sink.submit_command(
                            Selector::new("update_progress"), (1.0, 1.0, 1.0), None).unwrap();

                        let mut test_settings = settings.lock().unwrap();

                        test_settings.set("render_time", renderer.render_time as i64).unwrap();
                        test_settings.set("min_valid_iteration", renderer.series_approximation.min_valid_iteration as i64).unwrap();

                        event_sink.submit_command(
                            Selector::new("repaint"), (), None).unwrap();
                    }
                    "reset_renderer_fast" => {
                        let mut renderer = renderer_reference.lock().unwrap();

                        let total_pixels = (renderer.image_width * renderer.image_height) as f64;

                        let (tx, rx) = mpsc::channel();

                        let test = event_sink.clone();

                        let thread_counter_1 = renderer.progress.reference.clone();
                        let thread_counter_2 = renderer.progress.series_approximation.clone();
                        let thread_counter_3 = renderer.progress.reference_maximum.clone();
                        let thread_counter_4 = renderer.progress.series_validation.clone();
                        let thread_counter_5 = renderer.progress.iteration.clone();
                        let thread_counter_6 = renderer.progress.glitched_maximum.clone();

                        thread::spawn(move || {
                            loop {
                                match rx.try_recv() {
                                    Ok(_) => {
                                        break;
                                    },
                                    Err(_) => {
                                        // 40% weighting to first reference, 40% to SA calculation, 20% to SA checking
                                        let mut progress1 = 0.0;

                                        progress1 += 0.45 * thread_counter_1.get() as f64 / thread_counter_3.get() as f64;
                                        progress1 += 0.45 * thread_counter_2.get() as f64 / thread_counter_3.get() as f64;
                                        progress1 += 0.1 * thread_counter_4.get() as f64 / 2.0;

                                        // println!("reference: {}, series_approximation: {}, reference_maximum: {}", thread_counter_1.get(), thread_counter_2.get(), thread_counter_3.get());

                                        let glitched_amount = thread_counter_6.get();
                                        let complete_amount = total_pixels as f64 - glitched_amount as f64;

                                        let (progress2, progress3) = if glitched_amount != 0 {
                                            (1.0, (thread_counter_5.get() as f64 - complete_amount) / glitched_amount as f64)
                                        } else {
                                            (thread_counter_5.get() as f64 / total_pixels, 0.0)
                                        };
            
                                        test.submit_command(
                                            Selector::new("update_progress"), (progress1, progress2, progress3), None).unwrap();
                                    }
                                };
            
                                thread::sleep(Duration::from_millis(50));
                            };
                        });

                        renderer.render_frame(1, String::from(""));

                        tx.send(()).unwrap();

                        event_sink.submit_command(
                            Selector::new("update_progress"), (1.0, 1.0, 1.0), None).unwrap();

                        let mut test_settings = settings.lock().unwrap();

                        test_settings.set("render_time", renderer.render_time as i64).unwrap();
                        test_settings.set("min_valid_iteration", renderer.series_approximation.min_valid_iteration as i64).unwrap();

                        event_sink.submit_command(
                            Selector::new("repaint"), (), None).unwrap();
                    }
                    _ => {
                        println!("thread_command: {}", command);
                    }
                }
            }
            _ => {
                // wait 10ms before checking again
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}