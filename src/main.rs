use druid::widget::prelude::*;

use druid::{commands, AppLauncher, LocalizedString, Widget, WindowDesc, MouseButton, KeyCode, FileDialogOptions, FileSpec, Command};
use druid::piet::{Text, ImageFormat, InterpolationMode, TextLayoutBuilder, FontBuilder, Color};

use rust_fractal::renderer::FractalRenderer;
use rust_fractal::util::{ComplexFixed, ComplexExtended, FloatArbitrary, get_delta_top_left, extended_to_string_short};

use config::{Config, File};

struct FractalWidget {
    renderer: Option<FractalRenderer>,
    settings: Config
}

impl Widget<()> for FractalWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        ctx.request_focus();

        match event {
            Event::WindowConnected => {
                ctx.request_paint();
            },
            Event::MouseDown(e) => {
                if e.button == MouseButton::Left || e.button == MouseButton::Right {
                    // want window position
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

                            if e.button == MouseButton::Left {
                                zoom.mantissa *= 2.0;
                            } else {
                                zoom.mantissa /= 2.0;
                            }
                            zoom.reduce();

                            let mut location = renderer.center_reference.c.clone();
                            let precision = location.real().prec();

                            let temp = FloatArbitrary::with_val(precision, element.exponent).exp2();
                            let temp2 = FloatArbitrary::with_val(precision, element.mantissa.re);
                            let temp3 = FloatArbitrary::with_val(precision, element.mantissa.im);

                            *location.mut_real() += &temp2 * &temp;
                            *location.mut_imag() += &temp3 * &temp;

                            renderer.update_location(zoom, location);

                            renderer.render_frame(0, String::from(""));
                            ctx.request_paint();

                        },
                        None => {}
                    }; 
                }
            },
            Event::KeyUp(e) => {
                if e.key_code == KeyCode::KeyD {
                    let current_derivative = self.renderer.as_ref().unwrap().analytic_derivative;

                    self.settings.set("analytic_derivative", !current_derivative).unwrap();

                    self.renderer.as_mut().unwrap().analytic_derivative = !current_derivative;
                    self.renderer.as_mut().unwrap().data_export.analytic_derivative = !current_derivative;
                    self.renderer.as_mut().unwrap().render_frame(0, String::from(""));

                    ctx.request_paint();
                }

                if e.key_code == KeyCode::KeyO {
                    let toml = FileSpec::new("location", &["toml"]);

                    let open_dialog_options = FileDialogOptions::new()
                        .allowed_types(vec![toml]);

                    ctx.submit_command(Command::new(
                        druid::commands::SHOW_OPEN_PANEL,
                        open_dialog_options.clone(),
                    ), None);
                }

                if e.key_code == KeyCode::KeyP {
                    let toml = FileSpec::new("palette", &["toml"]);

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
                    self.settings.merge(File::with_name(file_info.path().to_str().unwrap())).unwrap();
                    self.renderer = None;
                }

                ctx.request_paint();
            },
            _ => {}
        }
        
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &(), _env: &Env) {
        
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {
        
    }

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        // BoxConstraints are passed by the parent widget.
        // This method can return any Size within those constraints:
        // bc.constrain(my_size)
        //
        // To check if a dimension is infinite or not (e.g. scrolling):
        // bc.is_width_bounded() / bc.is_height_bounded()
        bc.max()
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let size = ctx.size().to_rect();

        match &self.renderer {
            None => {
                self.settings.set("image_width", size.x1 as i64).unwrap();
                self.settings.set("image_height", size.y1 as i64).unwrap();

                self.renderer = Some(FractalRenderer::new(self.settings.clone()));
                self.renderer.as_mut().unwrap().render_frame(0, String::from(""));
            },
            _ => {}
        }

        let image = ctx
            .make_image(size.x1 as usize, size.y1 as usize, &self.renderer.as_ref().unwrap().data_export.rgb, ImageFormat::Rgb)
            .unwrap();
        // The image is automatically scaled to fit the rect you pass to draw_image
        ctx.draw_image(&image, size, InterpolationMode::Bilinear);

        let font = ctx.text()
            .new_font_by_name("Lucida Console", 24.0)
            .build()
            .unwrap();

        let layout = ctx.text()
            .new_text_layout(&font, &extended_to_string_short(self.renderer.as_ref().unwrap().zoom), std::f64::INFINITY)
            .build()
            .unwrap();
        
        ctx.draw_text(&layout, (10.0, 34.0), &Color::rgb8(0, 0, 0));
    }
}

pub fn main() {
    
    // settings.merge(File::with_name("e10000.toml")).unwrap();

    let window = WindowDesc::new(ui_builder).title(
        LocalizedString::new("rust-fractal"),
    ).window_size((1280.0, 720.0)).resizable(false);

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(())
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<()> {
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    FractalWidget {
        renderer: None,
        settings
    }
}