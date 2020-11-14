use std::sync::{Arc, Mutex};

use druid::widget::prelude::*;
use druid::widget::{Label, Split};
use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command, Data};
use druid::piet::{ImageFormat, InterpolationMode};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long};

use config::{Config, File};


struct FractalWidget {
    renderer: FractalRenderer,
}

#[derive(Clone, Data)]
struct FractalData {
    updated: usize,
    settings: Arc<Mutex<Config>>
}

impl FractalData {
    pub fn display(&self) -> String {
        let settings = self.settings.lock().unwrap();

        format!("zoom: {}\nreal: {}\nimag: {}\n{}x{}\niterations: {}\nderivative: {}\nrotate: {}\norder: {}\nskipped: {}\nrender_time: {}ms\nprobe_sampling: {}", 
            shorten_long_string(settings.get_str("zoom").unwrap().to_string()), 
            shorten_long_string(settings.get_str("real").unwrap().to_string()), 
            shorten_long_string(settings.get_str("imag").unwrap().to_string()),
            settings.get_int("image_width").unwrap(),
            settings.get_int("image_height").unwrap(),
            settings.get_int("iterations").unwrap(),
            settings.get_bool("analytic_derivative").unwrap(),
            settings.get_float("rotate").unwrap(),
            settings.get_int("approximation_order").unwrap(),
            settings.get_int("min_valid_iteration").unwrap(),
            settings.get_int("render_time").unwrap(),
            settings.get_int("probe_sampling").unwrap())
    }
}

fn shorten_long_string(string: String) -> String {
    let caps_string = string.to_ascii_uppercase();

    let values = caps_string.split("E").collect::<Vec<&str>>();

    let mut decimal = String::from(values[0]);
    decimal.truncate(6);

    if values.len() > 1 {
        format!("{}E{}", decimal, values[1])
    } else {
        format!("{}E0", decimal)
    }

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

                data.updated += 1;
            }
            Event::MouseDown(e) => {
                let mut settings = data.settings.lock().unwrap();

                // For a mousedown event we only check the left and right buttons
                if e.button == MouseButton::Left || e.button == MouseButton::Right {
                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();

                    // Zoom in, use the mouse position
                    if e.button == MouseButton::Left {
                        let size = ctx.size().to_rect();

                        let i = e.pos.x * self.renderer.image_width as f64 / size.width();
                        let j = e.pos.y * self.renderer.image_height as f64 / size.height();

                        println!("{}, {}", i, j);
    
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

                        // data.derive_from_settings(&self.current_settings, self.renderer.as_ref().unwrap());

                        self.renderer.update_location(zoom, location);
                        self.renderer.render_frame(0, String::from(""));
                    } else {
                        // Zoom out, only use the central location and save reference
                        self.renderer.zoom.mantissa /= 2.0;
                        self.renderer.zoom.reduce();

                        settings.set("zoom", extended_to_string_long(self.renderer.zoom)).unwrap();
                        // data.derive_from_settings(&self.current_settings, self.renderer.as_ref().unwrap());

                        // frame_index is set to 1 so that the reference is reused
                        self.renderer.render_frame(1, String::from(""));
                    }

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;

                    ctx.request_paint();
                }
            },
            Event::KeyUp(e) => {
                let mut settings = data.settings.lock().unwrap();

                if e.key_code == KeyCode::KeyD {

                    data.updated += 1;

                    let current_derivative = self.renderer.data_export.analytic_derivative;
                    settings.set("analytic_derivative", !current_derivative).unwrap();

                    self.renderer.data_export.analytic_derivative = !current_derivative;

                    // We have already computed the iterations and analytic derivatives
                    if self.renderer.analytic_derivative {
                        self.renderer.data_export.regenerate();
                    } else {
                        self.renderer.analytic_derivative = true;
                        self.renderer.render_frame(1, String::from(""));
                    }

                    // Toggle the use of the analytic derivative
                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyZ {
                    data.updated += 1;

                    self.renderer.zoom.mantissa *= 2.0;
                    self.renderer.zoom.reduce();

                    settings.set("zoom", extended_to_string_long(self.renderer.zoom)).unwrap();
                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyO {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let open_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_OPEN_PANEL,
                        open_dialog_options.clone(),
                    ), None);
                }

                if e.key_code == KeyCode::KeyN {
                    let window_width = settings.get_float("window_width").unwrap();
                    let window_height = settings.get_float("window_height").unwrap();

                    settings.set("image_width", window_width as i64).unwrap();
                    settings.set("image_height", window_height as i64).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.image_width = window_width as usize;
                    self.renderer.image_height = window_height as usize;
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyT {
                    let new_width = settings.get_int("image_width").unwrap() / 2;
                    let new_height = settings.get_int("image_height").unwrap() / 2;

                    settings.set("image_width", new_width).unwrap();
                    settings.set("image_height", new_height).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.image_width = new_width as usize;
                    self.renderer.image_height = new_height as usize;
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyY {
                    let new_width = settings.get_int("image_width").unwrap() * 2;
                    let new_height = settings.get_int("image_height").unwrap() * 2;

                    settings.set("image_width", new_width).unwrap();
                    settings.set("image_height", new_height).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.image_width = new_width as usize;
                    self.renderer.image_height = new_height as usize;
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;
                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyR {
                    let new_rotate = (settings.get_float("rotate").unwrap() + 5.0) % 360.0;

                    settings.set("rotate", new_rotate).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.rotate = new_rotate.to_radians();

                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;
                    ctx.request_paint();
                }

                // TODO make this so that the reference is not reset
                // if e.key_code == KeyCode::KeyP {
                //     let mut new_probes = settings.get_int("probe_sampling").unwrap() / 2;

                //     if new_probes < 2 {
                //         new_probes = 2;
                //     }

                //     settings.set("probe_sampling", new_probes).unwrap();

                //     self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                //     self.renderer.render_frame(1, String::from(""));

                //     settings.set("render_time", self.renderer.render_time as i64).unwrap();
                //     settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                //     data.updated += 1;
                //     ctx.request_paint();
                // }
            },
            Event::Command(command) => {
                // if command.is::<()>(Selector::new("refresh_data")) {
                //     data.updated += 1;
                // } else {
                let mut settings = data.settings.lock().unwrap();

                if let Some(file_info) = command.get(commands::OPEN_FILE) {
                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    let mut reset_renderer = false;

                    match new_settings.get_str("real") {
                        Ok(real) => {
                            settings.set("real", real).unwrap();
                            settings.set("rotate", 0.0).unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("imag") {
                        Ok(imag) => {
                            settings.set("imag", imag).unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("zoom") {
                        Ok(zoom) => {
                            settings.set("zoom", zoom).unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_int("iterations") {
                        Ok(iterations) => {
                            settings.set("iterations", iterations).unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_float("rotate") {
                        Ok(rotate) => {
                            settings.set("rotate", rotate).unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    if reset_renderer {
                        self.renderer = FractalRenderer::new(settings.clone());
                    }

                    self.renderer.render_frame(0, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();
                    
                    match new_settings.get_float("iteration_division") {
                        Ok(iteration_division) => {
                            settings.set("iteration_division", iteration_division).unwrap();
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_array("palette") {
                        Ok(colour_values) => {
                            settings.set("palette", colour_values.clone()).unwrap();

                            let palette = colour_values.chunks_exact(3).map(|value| {
                                // We assume the palette is in BGR rather than RGB
                                (value[2].clone().into_int().unwrap() as u8, 
                                    value[1].clone().into_int().unwrap() as u8, 
                                    value[0].clone().into_int().unwrap() as u8)
                            }).collect::<Vec<(u8, u8, u8)>>();

                            self.renderer.data_export.palette = palette;
                            self.renderer.data_export.iteration_division = settings.get_float("iteration_division").unwrap() as f32;

                            self.renderer.data_export.regenerate();
                        }
                        Err(_) => {}
                    }

                    settings.merge(new_settings).unwrap();

                    data.updated += 1;

                    // data.derive_from_settings(&self.current_settings, self.renderer.as_ref().unwrap());
                    ctx.request_paint();
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

    let window = WindowDesc::new(ui_builder).title(
        LocalizedString::new("rust-fractal"),
    ).window_size((1280.0, 720.0)).resizable(true);

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(FractalData {
            updated: 0,
            settings: Arc::new(Mutex::new(settings))
        })
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<FractalData> {
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    let render_screen = FractalWidget {
        renderer: FractalRenderer::new(settings.clone()),
        // current_settings: settings,
    };

    let mut label = Label::new(|data: &FractalData, _env: &_| {
        data.display()
    });

    label.set_text_size(20.0);
    label.set_font("Lucida Console".to_string());

    Split::columns(render_screen, label).split_point(0.8).draggable(true)
}