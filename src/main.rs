use std::sync::{Arc, Mutex};

use druid::widget::prelude::*;

use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command, Data, Lens, Selector};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::theme::{BUTTON_BORDER_RADIUS, TEXT_SIZE_NORMAL, FONT_NAME, TEXTBOX_BORDER_RADIUS};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long, string_to_extended};

use config::{Config, File};

mod ui;
pub mod lens;

struct FractalWidget {
    renderer: FractalRenderer,
}

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
    temporary_progress: f64,
    settings: Arc<Mutex<Config>>
}

impl Widget<FractalData> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FractalData, _env: &Env) {
        ctx.request_focus();

        match event {
            Event::WindowConnected => {
                let mut settings = data.settings.lock().unwrap();

                self.renderer.render_frame(0, String::from(""));
                settings.set("render_time", self.renderer.render_time as i64).unwrap();
                settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                data.temporary_width = settings.get_int("image_width").unwrap();
                data.temporary_height = settings.get_int("image_height").unwrap();
                data.updated += 1;
            }
            Event::MouseDown(e) => {
                let mut settings = data.settings.lock().unwrap();

                // For a mousedown event we only check the left and right buttons
                if e.button == MouseButton::Left || e.button == MouseButton::Right {
                    // Zoom in, use the mouse position
                    if e.button == MouseButton::Left {
                        let size = ctx.size().to_rect();

                        let i = e.pos.x * self.renderer.image_width as f64 / size.width();
                        let j = e.pos.y * self.renderer.image_height as f64 / size.height();
    
                        let cos_rotate = self.renderer.rotate.cos();
                        let sin_rotate = self.renderer.rotate.sin();
    
                        let delta_pixel =  4.0 / ((self.renderer.image_height - 1) as f64 * self.renderer.zoom.mantissa);
                        let delta_top_left = get_delta_top_left(delta_pixel, self.renderer.image_width, self.renderer.image_height, cos_rotate, sin_rotate);
    
                        let element = ComplexFixed::new(
                            i * delta_pixel * cos_rotate - j * delta_pixel * sin_rotate + delta_top_left.re, 
                            i * delta_pixel * sin_rotate + j * delta_pixel * cos_rotate + delta_top_left.im
                        );

                        let element = ComplexExtended::new(element, -self.renderer.zoom.exponent);
                        let mut zoom = self.renderer.zoom;
                    
                        zoom.mantissa *= 2.0;
                        zoom.reduce();

                        let mut location = self.renderer.center_reference.c.clone();
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
                        self.renderer.maximum_iteration = settings.get_int("iterations").unwrap() as usize;
                        self.renderer.update_location(zoom, location);

                        // BUG, somewhere in this update thing, need to deal with if the maximum iteration is less than reference or something
                        settings.set("iterations", self.renderer.maximum_iteration as i64).unwrap();
                        data.temporary_iterations = self.renderer.maximum_iteration as i64;

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
                let mut settings = data.settings.lock().unwrap();

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
                    if dimensions.0 as usize == self.renderer.image_width && dimensions.1 as usize == self.renderer.image_height {
                        return;
                    }

                    settings.set("image_width", dimensions.0 as i64).unwrap();
                    settings.set("image_height", dimensions.1 as i64).unwrap();

                    self.renderer.image_width = dimensions.0 as usize;
                    self.renderer.image_height = dimensions.1 as usize;

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(iterations) = command.get::<i64>(Selector::new("set_iterations")) {
                    if *iterations as usize == self.renderer.data_export.maximum_iteration {
                        return;
                    }

                    settings.set("iterations", *iterations).unwrap();
                    data.temporary_iterations = *iterations;

                    if *iterations as usize <= self.renderer.maximum_iteration {
                        self.renderer.data_export.maximum_iteration = data.temporary_iterations as usize;
                        self.renderer.data_export.regenerate();

                        data.updated += 1;
                        ctx.request_paint();
                        return;
                    }

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("set_approximation_order")) {
                    if (data.temporary_order as usize) == self.renderer.series_approximation.order {
                        return;
                    }

                    if (data.temporary_order as usize) > 128 {
                        data.temporary_order = 128;
                    }

                    if (data.temporary_order as usize) < 4 {
                        data.temporary_order = 4;
                    }

                    settings.set("approximation_order", data.temporary_order).unwrap();
                    self.renderer.series_approximation.order = data.temporary_order as usize;

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

                            if (data.temporary_iterations as usize) < self.renderer.maximum_iteration {
                                // TODO needs to make it so that pixels are only iterated to the right level
                                self.renderer.maximum_iteration = data.temporary_iterations as usize;
                                ctx.submit_command(Command::new(Selector::new("set_rotation"), data.temporary_rotation), None);
                                return;
                            }
                        } else {
                            // Zoom has changed, and need to rerender depending on if the zoom has changed too much

                            // Look at something like this for the renderer
                            // https://github.com/linebender/druid/blob/master/druid/examples/async_event.rs

                            let current_exponent = self.renderer.center_reference.zoom.exponent;
                            let new_zoom = string_to_extended(&data.temporary_zoom.to_uppercase());

                            if new_zoom.exponent <= current_exponent {
                                println!("zoom decreased");
                                self.renderer.zoom = new_zoom;
                                settings.set("zoom", data.temporary_zoom.clone()).unwrap();

                                self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                                self.renderer.render_frame(1, String::from(""));

                                settings.set("render_time", self.renderer.render_time as i64).unwrap();
                                settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                                data.updated += 1;
                                ctx.request_paint();
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
                    self.renderer.zoom.mantissa *= factor;
                    self.renderer.zoom.reduce();

                    settings.set("zoom", extended_to_string_long(self.renderer.zoom)).unwrap();
                    data.temporary_zoom = settings.get_str("zoom").unwrap();

                    // TODO properly set the maximum iterations
                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("toggle_derivative")) {
                    let current_derivative = self.renderer.data_export.analytic_derivative;
                    settings.set("analytic_derivative", !current_derivative).unwrap();
                    self.renderer.data_export.analytic_derivative = !current_derivative;

                    // We have already computed the iterations and analytic derivatives
                    if self.renderer.analytic_derivative {
                        self.renderer.data_export.regenerate();
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

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.rotate = new_rotate.to_radians();

                    ctx.submit_command(Command::new(Selector::new("reset_renderer_fast"), ()), None);
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("set_offset_division")) {
                    let current_division = settings.get_float("iteration_division").unwrap() as f32;
                    let current_offset = settings.get_float("palette_offset").unwrap() as f32;

                    let new_division = data.temporary_iteration_division.parse::<f32>().unwrap();
                    let new_offset = data.temporary_iteration_offset.parse::<f32>().unwrap() % self.renderer.data_export.palette.len() as f32;

                    println!("{} {} {}", data.temporary_iteration_offset, new_offset, new_division);

                    if current_division == new_division && current_offset == new_offset {
                        return;
                    }

                    data.temporary_iteration_division = new_division.to_string();
                    data.temporary_iteration_offset = new_offset.to_string();

                    settings.set("iteration_division", new_division as f64).unwrap();
                    settings.set("palette_offset", new_offset as f64).unwrap();

                    self.renderer.data_export.iteration_division = new_division;
                    self.renderer.data_export.iteration_offset = new_offset;

                    self.renderer.data_export.regenerate();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("reset_renderer_fast")) {
                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                    return;
                }

                if let Some(_) = command.get::<()>(Selector::new("reset_renderer_full")) {
                    data.temporary_progress = 0.5;

                    ctx.request_paint();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer = FractalRenderer::new(settings.clone());
                    self.renderer.render_frame(0, String::from(""));

                    // data.temporary_progress = 0.5;


                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.temporary_order = settings.get_int("approximation_order").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
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

                            self.renderer.data_export.palette = palette;
                            self.renderer.data_export.iteration_division = settings.get_float("iteration_division").unwrap() as f32;
                            self.renderer.data_export.iteration_offset = settings.get_float("palette_offset").unwrap() as f32;

                            data.temporary_palette_source = file_info.path().file_name().unwrap().to_str().unwrap().to_string();

                            if !reset_renderer {
                                self.renderer.data_export.regenerate();
                            }
                        }
                        Err(_) => {}
                    }

                    settings.merge(new_settings).unwrap();

                    if reset_renderer {
                        self.renderer = FractalRenderer::new(settings.clone());
                        self.renderer.render_frame(0, String::from(""));

                        settings.set("render_time", self.renderer.render_time as i64).unwrap();
                        settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();
                    }

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                    return;
                }

                if let Some(file_info) = command.get(commands::SAVE_FILE) {
                    match file_info.clone().unwrap().path().extension().unwrap().to_str().unwrap() {
                        "png" | "jpg" => {
                            self.renderer.data_export.save_colour(file_info.clone().unwrap().path().to_str().unwrap());
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

        test.height = test.width * self.renderer.image_height as f64 / self.renderer.image_width as f64;

        test
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &FractalData, _env: &Env) {
        let size = ctx.size().to_rect();

        let image = ctx
            .make_image(self.renderer.image_width, self.renderer.image_height, &self.renderer.data_export.rgb, ImageFormat::Rgb)
            .unwrap();

        if self.renderer.image_width > size.width() as usize {
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

    AppLauncher::with_window(window)
        // .use_simple_logger()
        .configure_env(|env, _| {
            env.set(FONT_NAME, "Lucida Console");
            env.set(TEXT_SIZE_NORMAL, 12.0);
            env.set(BUTTON_BORDER_RADIUS, 2.0);
            env.set(TEXTBOX_BORDER_RADIUS, 2.0);
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
            temporary_progress: 0.0,
            settings: Arc::new(Mutex::new(settings))
        })
        .expect("launch failed");
}