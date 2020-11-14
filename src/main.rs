use std::sync::{Arc, Mutex};

use druid::widget::prelude::*;
use druid::widget::{Label, Split, TextBox, Flex, Button, CrossAxisAlignment};
use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command, Data, Lens, LensWrap, Selector};
use druid::piet::{ImageFormat, InterpolationMode};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long};

use config::{Config, File};


struct FractalWidget {
    renderer: FractalRenderer,
}

#[derive(Clone, Data, Lens)]
struct FractalData {
    updated: usize,
    temporary_width: i64,
    temporary_height: i64,
    settings: Arc<Mutex<Config>>
}

struct WidthLens;

impl Lens<FractalData, String> for WidthLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let mut string = data.temporary_width.to_string();
        if data.temporary_width == 0 {
            string = "".into();
        }
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_width.to_string();
        if data.temporary_width == 0 {
            string = "".into();
        }
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_width = string.parse().unwrap_or(0);
        v
    }
}

struct HeightLens;

impl Lens<FractalData, String> for HeightLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let mut string = data.temporary_height.to_string();
        if data.temporary_height == 0 {
            string = "".into();
        }
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_height.to_string();
        if data.temporary_height == 0 {
            string = "".into();
        }
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_height = string.parse().unwrap_or(0);
        v
    }
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

                data.temporary_width = settings.get_int("image_width").unwrap();
                data.temporary_height = settings.get_int("image_height").unwrap();
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

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
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

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
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

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyO {
                    let toml = FileSpec::new("configuration", &["toml"]);

                    let open_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_OPEN_PANEL,
                        open_dialog_options.clone(),
                    ), None);
                }

                if e.key_code == KeyCode::KeyN {
                    ctx.submit_command(Command::new(Selector::new("native_image_size"), ()), None);
                }

                if e.key_code == KeyCode::KeyT {
                    ctx.submit_command(Command::new(Selector::new("half_image_size"), ()), None);
                }

                if e.key_code == KeyCode::KeyY {
                    ctx.submit_command(Command::new(Selector::new("double_image_size"), ()), None);
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
                println!("{:?}", command);
                let mut settings = data.settings.lock().unwrap();

                if let Some(_) = command.get::<()>(Selector::new("half_image_size")) {
                    let new_width = settings.get_int("image_width").unwrap() / 2;
                    let new_height = settings.get_int("image_height").unwrap() / 2;

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (new_width, new_height)), None);
                }

                if let Some(_) = command.get::<()>(Selector::new("double_image_size")) {
                    let new_width = settings.get_int("image_width").unwrap() * 2;
                    let new_height = settings.get_int("image_height").unwrap() * 2;

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (new_width, new_height)), None);
                }

                if let Some(_) = command.get::<()>(Selector::new("native_image_size")) {
                    let window_width = settings.get_float("window_width").unwrap();
                    let window_height = settings.get_float("window_height").unwrap();

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (window_width as i64, window_height as i64)), None);
                }

                if let Some(dimensions) = command.get::<(i64, i64)>(Selector::new("set_image_size")) {
                    settings.set("image_width", dimensions.0 as i64).unwrap();
                    settings.set("image_height", dimensions.1 as i64).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.image_width = dimensions.0 as usize;
                    self.renderer.image_height = dimensions.1 as usize;
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;
                    ctx.request_paint();
                    return;
                }

                // if command.is::<()>(Selector::new("refresh_data")) {
                //     data.updated += 1;
                // } else {

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
            temporary_width: settings.get_int("image_width").unwrap(),
            temporary_height: settings.get_int("image_height").unwrap(),
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
        println!("update!");
        data.display()
    });

    label.set_text_size(20.0);
    label.set_font("Lucida Console".to_string());

    let image_width = LensWrap::new(TextBox::new(), WidthLens);
    let image_height = LensWrap::new(TextBox::new(), HeightLens);

    let button_set = Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_image_size"), (data.temporary_width, data.temporary_height)), None);
    });

    let button_half = Button::new("HALF").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("half_image_size"), ()), None);
    });

    let button_double = Button::new("DOUBLE").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("double_image_size"), ()), None);
    });

    let button_native = Button::new("NATIVE").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("native_image_size"), ()), None);
    });

    let mut row_1 = Label::<FractalData>::new("IMAGE RESOLUTION");

    row_1.set_text_size(20.0);
    row_1.set_font("Lucida Console".to_string());

    let mut width_label = Label::<FractalData>::new("WIDTH: ");
    let mut height_label = Label::<FractalData>::new("HEIGHT: ");

    width_label.set_text_size(20.0);
    width_label.set_font("Lucida Console".to_string());

    height_label.set_text_size(20.0);
    height_label.set_font("Lucida Console".to_string());

    let mut row_2 = Flex::row()
        .with_flex_child(width_label, 1.0)
        .with_spacer(10.0)
        .with_child(image_width);

    // row_2.must_fill_main_axis();
        
    let row_3 = Flex::row()
        .with_child(height_label)
        .with_spacer(10.0)
        .with_child(image_height);

    let row_4 = Flex::row()
        .with_child(button_set)
        .with_child(button_half)
        .with_child(button_double)
        .with_child(button_native);

    // let image_size_layout = Flex::row()
    //     .with_child(image_width)
    //     .with_child(image_height)
    //     .with_child(button_set);

    let flex = Flex::<FractalData>::column()
        .with_child(row_1)
        .with_child(row_2)
        .with_child(row_3)
        .with_child(row_4)
        .with_child(label);


    Split::columns(render_screen, flex).split_point(0.8).draggable(true).min_size(100.0)
}