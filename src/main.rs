use std::sync::{Arc, Mutex};

use druid::widget::prelude::*;
use druid::widget::{Label, Split, TextBox, Flex, Button, WidgetExt};
use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command, Data, Lens, LensWrap, Selector};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::theme::{BUTTON_BORDER_RADIUS, TEXT_SIZE_NORMAL, FONT_NAME};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_long};

use config::{Config, File};

mod lens;

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
                        self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
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

                        data.temporary_real = settings.get_str("real").unwrap();
                        data.temporary_imag = settings.get_str("imag").unwrap();
                        data.temporary_zoom = settings.get_str("zoom").unwrap();

                        // data.derive_from_settings(&self.current_settings, self.renderer.as_ref().unwrap());

                        self.renderer.update_location(zoom, location);
                        self.renderer.render_frame(0, String::from(""));

                        settings.set("render_time", self.renderer.render_time as i64).unwrap();
                        settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                        data.temporary_width = settings.get_int("image_width").unwrap();
                        data.temporary_height = settings.get_int("image_height").unwrap();
                        data.updated += 1;

                        ctx.request_paint();
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
                println!("{:?}", command);
                let mut settings = data.settings.lock().unwrap();

                if let Some(factor) = command.get::<f64>(Selector::new("multiply_image_size")) {
                    let new_width = settings.get_int("image_width").unwrap() as f64 * factor;
                    let new_height = settings.get_int("image_height").unwrap() as f64 * factor;

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (new_width as i64, new_height as i64)), None);
                }

                if let Some(_) = command.get::<()>(Selector::new("native_image_size")) {
                    let window_width = settings.get_float("window_width").unwrap();
                    let window_height = settings.get_float("window_height").unwrap();

                    ctx.submit_command(Command::new(Selector::new("set_image_size"), (window_width as i64, window_height as i64)), None);
                }

                if let Some(dimensions) = command.get::<(i64, i64)>(Selector::new("set_image_size")) {
                    if dimensions.0 as usize == self.renderer.image_width && dimensions.1 as usize == self.renderer.image_height {
                        return;
                    }

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

                if let Some(_) = command.get::<()>(Selector::new("set_location")) {
                    let current_real = settings.get_str("real").unwrap();
                    let current_imag = settings.get_str("imag").unwrap();
                    let current_zoom = settings.get_str("zoom").unwrap();

                    if (current_real == data.temporary_real && current_imag == data.temporary_imag) && current_zoom == data.temporary_zoom {
                        return;
                    }

                    settings.set("real", data.temporary_real.clone()).unwrap();
                    settings.set("imag", data.temporary_imag.clone()).unwrap();
                    settings.set("zoom", data.temporary_zoom.clone()).unwrap();
                    settings.set("rotate", 0.0).unwrap();

                    self.renderer = FractalRenderer::new(settings.clone());
                    self.renderer.render_frame(0, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                }

                if let Some(factor) = command.get::<f64>(Selector::new("multiply_zoom_level")) {
                    self.renderer.zoom.mantissa *= factor;
                    self.renderer.zoom.reduce();

                    settings.set("zoom", extended_to_string_long(self.renderer.zoom)).unwrap();
                    data.temporary_zoom = settings.get_str("zoom").unwrap();
                    
                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.temporary_width = settings.get_int("image_width").unwrap();
                    data.temporary_height = settings.get_int("image_height").unwrap();
                    data.updated += 1;

                    ctx.request_paint();
                }

                if let Some(_) = command.get::<()>(Selector::new("toggle_derivative")) {
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

                if let Some(rotation) = command.get::<f64>(Selector::new("set_rotation")) {
                    let new_rotate = rotation % 360.0;

                    settings.set("rotate", new_rotate).unwrap();

                    self.renderer.analytic_derivative = settings.get("analytic_derivative").unwrap();
                    self.renderer.rotate = new_rotate.to_radians();

                    self.renderer.render_frame(1, String::from(""));

                    settings.set("render_time", self.renderer.render_time as i64).unwrap();
                    settings.set("min_valid_iteration", self.renderer.series_approximation.min_valid_iteration as i64).unwrap();

                    data.updated += 1;
                    ctx.request_paint();
                }

                if let Some(file_info) = command.get(commands::OPEN_FILE) {
                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    let mut reset_renderer = false;

                    match new_settings.get_str("real") {
                        Ok(real) => {
                            settings.set("real", real).unwrap();
                            settings.set("rotate", 0.0).unwrap();
                            data.temporary_real = settings.get_str("real").unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("imag") {
                        Ok(imag) => {
                            settings.set("imag", imag).unwrap();
                            data.temporary_imag = settings.get_str("imag").unwrap();
                            reset_renderer = true;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("zoom") {
                        Ok(zoom) => {
                            settings.set("zoom", zoom).unwrap();
                            data.temporary_zoom = settings.get_str("zoom").unwrap();
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
        .configure_env(|env, _| {
            env.set(FONT_NAME, "Lucida Console");
            env.set(TEXT_SIZE_NORMAL, 12.0);
            env.set(BUTTON_BORDER_RADIUS, 0.0);

            for test in env.get_all() {
                println!("{:?}", test);
            };

            


        })
        .launch(FractalData {
            updated: 0,
            temporary_width: settings.get_int("image_width").unwrap(),
            temporary_height: settings.get_int("image_height").unwrap(),
            temporary_real: settings.get_str("real").unwrap(),
            temporary_imag: settings.get_str("imag").unwrap(),
            temporary_zoom: settings.get_str("zoom").unwrap(),
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

    let image_width = LensWrap::new(TextBox::new().expand_width(), lens::WidthLens);
    let image_height = LensWrap::new(TextBox::new().expand_width(), lens::HeightLens);

    let button_set = Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_image_size"), (data.temporary_width, data.temporary_height)), None);
    }).expand_width();

    let button_half = Button::new("HALF").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 0.5), None);
    }).expand_width();

    let button_double = Button::new("DOUBLE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 2.0), None);
    }).expand_width();

    let button_native = Button::new("NATIVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("native_image_size"), ()), None);
    }).expand_width();

    let mut resolution_title = Label::<FractalData>::new("RESOLUTION");
    resolution_title.set_text_size(20.0);

    let row_1 = Flex::row()
        .with_flex_child(resolution_title.expand_width(), 1.0);

    let mut width_label = Label::<FractalData>::new("WIDTH:  ");
    let mut height_label = Label::<FractalData>::new("HEIGHT: ");

    width_label.set_text_size(20.0);
    height_label.set_text_size(20.0);

    let row_2 = Flex::row()
        .with_child(width_label)
        .with_flex_child(image_width, 1.0);

    let row_3 = Flex::row()
        .with_child(height_label)
        .with_flex_child(image_height, 1.0);

    let row_4 = Flex::row()
        .with_flex_child(button_set, 1.0)
        .with_flex_child(button_half, 1.0)
        .with_flex_child(button_double, 1.0)
        .with_flex_child(button_native, 1.0);

    let mut location_title = Label::<FractalData>::new("LOCATION");
    location_title.set_text_size(20.0);

    let row_5 = Flex::row()
        .with_flex_child(location_title.expand_width(), 1.0);

    let mut real_label = Label::<FractalData>::new("REAL: ");
    let mut imag_label = Label::<FractalData>::new("IMAG: ");
    let mut zoom_label = Label::<FractalData>::new("ZOOM: ");

    real_label.set_text_size(20.0);
    imag_label.set_text_size(20.0);
    zoom_label.set_text_size(20.0);

    let real = LensWrap::new(TextBox::new().expand_width(), lens::RealLens);
    let imag = LensWrap::new(TextBox::new().expand_width(), lens::ImagLens);
    let zoom = LensWrap::new(TextBox::new().expand_width(), lens::ZoomLens);

    let row_6 = Flex::row()
        .with_child(real_label)
        .with_flex_child(real, 1.0);

    let row_7 = Flex::row()
        .with_child(imag_label)
        .with_flex_child(imag, 1.0);

    let row_8 = Flex::row()
        .with_child(zoom_label)
        .with_flex_child(zoom, 1.0);

    let button_set_location = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_location"), ()), None);
    }).expand_width();

    let button_zoom_in = Button::new("IN").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 2.0), None);
    }).expand_width();

    let button_zoom_out = Button::new("OUT").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 0.5), None);
    }).expand_width();

    let button_load_location = Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("open_location"), ()), None);
    }).expand_width();

    let button_save_location = Button::new("SAVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("double_image_size"), ()), None);
    }).expand_width();

    let row_9 = Flex::row()
        .with_flex_child(button_set_location, 1.0)
        .with_flex_child(button_zoom_in, 1.0)
        .with_flex_child(button_zoom_out, 1.0)
        .with_flex_child(button_load_location, 1.0)
        .with_flex_child(button_save_location, 1.0);

    let mut columns = Flex::<FractalData>::column()
        .with_spacer(8.0)
        .with_child(row_1)
        .with_child(row_2)
        .with_child(row_3)
        .with_child(row_4)
        .with_spacer(8.0)
        .with_child(row_5)
        .with_child(row_6)
        .with_child(row_7)
        .with_child(row_8)
        .with_child(row_9)
        .with_spacer(8.0)
        .with_child(label);

    columns.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    let mut flex = Flex::<FractalData>::row()
        .with_flex_spacer(0.1)
        .with_flex_child(columns, 0.8)
        .with_flex_spacer(0.1);
    
    flex.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);


    Split::columns(render_screen, flex).split_point(0.75).draggable(true)
}