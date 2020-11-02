use druid::widget::prelude::*;
use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command};
use druid::piet::{Text, ImageFormat, InterpolationMode, TextLayoutBuilder, FontBuilder, Color};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_short, extended_to_string_long};

use config::{Config, File};

struct FractalWidget {
    renderer: Option<FractalRenderer>,
    current_settings: Config
}

impl Widget<()> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        // This is used so that the keyboard commands will work
        ctx.request_focus();

        match event {
            Event::WindowConnected => {
                ctx.request_paint();
            },
            Event::MouseDown(e) => {
                // For a mousedown event we only check the left and right buttons
                if e.button == MouseButton::Left || e.button == MouseButton::Right {
                    match &mut self.renderer {
                        Some(renderer) => {
                            let i = e.pos.x;
                            let j = e.pos.y;
        
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

                            renderer.analytic_derivative = self.current_settings.get("analytic_derivative").unwrap();

                            // Zoom in, use the mouse position
                            if e.button == MouseButton::Left {
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
                                self.current_settings.set("real", location.real().to_string()).unwrap();
                                self.current_settings.set("imag", location.imag().to_string()).unwrap();
                                self.current_settings.set("zoom", extended_to_string_long(zoom)).unwrap();

                                renderer.update_location(zoom, location);
                                renderer.render_frame(0, String::from(""));
                            } else {
                                // Zoom out, only use the central location and save reference
                                renderer.zoom.mantissa /= 2.0;
                                renderer.zoom.reduce();

                                // frame_index is set to 1 so that the reference is reused
                                renderer.render_frame(1, String::from(""));
                            }

                            ctx.request_paint();

                        },
                        None => {}
                    }; 
                }
            },
            Event::KeyUp(e) => {
                if e.key_code == KeyCode::KeyD {
                    let renderer = self.renderer.as_mut().unwrap();

                    let current_derivative = renderer.data_export.analytic_derivative;
                    self.current_settings.set("analytic_derivative", !current_derivative).unwrap();

                    renderer.data_export.analytic_derivative = !current_derivative;

                    // We have already computed the iterations and analytic derivatives
                    if renderer.analytic_derivative {
                        renderer.data_export.regenerate();
                    } else {
                        renderer.analytic_derivative = true;
                        renderer.render_frame(1, String::from(""));
                    }

                    // Toggle the use of the analytic derivative

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyZ {
                    let renderer = self.renderer.as_mut().unwrap();

                    renderer.zoom.mantissa *= 2.0;
                    renderer.zoom.reduce();

                    renderer.analytic_derivative = self.current_settings.get("analytic_derivative").unwrap();
                    renderer.render_frame(1, String::from(""));
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
            },
            Event::Command(command) => {
                if let Some(file_info) = command.get(commands::OPEN_FILE) {
                    let mut new_settings = Config::default();
                    new_settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();

                    match new_settings.get_str("real") {
                        Ok(real) => {
                            self.current_settings.set("real", real).unwrap();
                            self.renderer = None;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("imag") {
                        Ok(imag) => {
                            self.current_settings.set("imag", imag).unwrap();
                            self.renderer = None;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_str("zoom") {
                        Ok(zoom) => {
                            self.current_settings.set("zoom", zoom).unwrap();
                            self.renderer = None;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_float("rotate") {
                        Ok(rotate) => {
                            self.current_settings.set("rotate", rotate).unwrap();
                            self.renderer = None;
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_float("iteration_division") {
                        Ok(iteration_division) => {
                            self.current_settings.set("iteration_division", iteration_division).unwrap();
                        }
                        Err(_) => {}
                    }

                    match new_settings.get_array("palette") {
                        Ok(colour_values) => {
                            self.current_settings.set("palette", colour_values.clone()).unwrap();

                            match &mut self.renderer {
                                Some(renderer) => {
                                    let palette = colour_values.chunks_exact(3).map(|value| {
                                        // We assume the palette is in BGR rather than RGB
                                        (value[2].clone().into_int().unwrap() as u8, 
                                            value[1].clone().into_int().unwrap() as u8, 
                                            value[0].clone().into_int().unwrap() as u8)
                                    }).collect::<Vec<(u8, u8, u8)>>();

                                    renderer.data_export.palette = palette;
                                    renderer.data_export.iteration_division = self.current_settings.get_float("iteration_division").unwrap() as f32;

                                    renderer.data_export.regenerate();
                                },
                                None => {}
                            }
                        }
                        Err(_) => {}
                    }

                    self.current_settings.merge(new_settings).unwrap();
                }

                ctx.request_paint();
            },
            _ => {}
        }
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &(), _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {}

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let size = ctx.size().to_rect();

        match &self.renderer {
            None => {
                self.current_settings.set("image_width", size.x1 as i64).unwrap();
                self.current_settings.set("image_height", size.y1 as i64).unwrap();

                let max_iterations = self.current_settings.get_int("iterations").unwrap();

                if max_iterations > 10000 {
                    self.current_settings.set("approximation_order", 64).unwrap();
                }

                self.renderer = Some(FractalRenderer::new(self.current_settings.clone()));
                self.renderer.as_mut().unwrap().render_frame(0, String::from(""));
            },
            _ => {}
        }

        let image = ctx
            .make_image(size.x1 as usize, size.y1 as usize, &self.renderer.as_ref().unwrap().data_export.rgb, ImageFormat::Rgb)
            .unwrap();

        ctx.draw_image(&image, size, InterpolationMode::Bilinear);

        let font = ctx.text()
            .new_font_by_name("Lucida Console", 20.0)
            .build()
            .unwrap();

        let renderer = self.renderer.as_ref().unwrap();

        let colouring_type = if self.current_settings.get("analytic_derivative").unwrap() {
            "Distance"
        } else {
            "Iteration"
        };

        let layout = ctx.text()
            .new_text_layout(
                &font, 
                &format!("Zoom: {}\nMaximum: {}\nSkipped: {}\nOrder: {}\nColouring: {}\nElapsed: {}ms", 
                    extended_to_string_short(renderer.zoom), 
                    renderer.center_reference.maximum_iteration, 
                    renderer.series_approximation.min_valid_iteration, 
                    renderer.series_approximation.order,
                    colouring_type,
                    renderer.render_time), 
                std::f64::INFINITY)
            .build()
            .unwrap();
        
        ctx.draw_text(&layout, (6.0, 20.0), &Color::rgb8(0, 0, 0));
    }
}

pub fn main() {
    let window = WindowDesc::new(ui_builder).title(
        LocalizedString::new("rust-fractal"),
    ).window_size((1280.0, 720.0)).resizable(false);

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(())
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<()> {
    // Setup the default settings. These are stored in start.toml file
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    FractalWidget {
        renderer: None,
        current_settings: settings,
    }
}