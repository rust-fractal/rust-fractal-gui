use std::{fmt::Display, str::FromStr};
use druid::{commands::CLOSE_WINDOW, 
    widget::{Align, Button, 
        Checkbox, CrossAxisAlignment, FillStrat, Flex, Image, Label, ProgressBar, Slider, Split, TextBox, WidgetExt, Painter}, 
    Command, Target, RenderContext};
use druid::{Widget, ImageBuf, Data, LensExt, Menu, LocalizedString, MenuItem, SysMods, Env, WindowId, WindowDesc};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::text::ParseFormatter;
use druid::commands::CLOSE_ALL_WINDOWS;

use druid::theme::{PRIMARY_DARK, BACKGROUND_DARK};

use parking_lot::Mutex;
use std::sync::Arc;
use rust_fractal::{
    renderer::FractalRenderer,
    util::{string_to_extended, extended_to_string_short, FloatExtended}
};

use crate::{FractalData, FractalWidget, ColoringType};
use crate::custom::*;


use crate::commands::*;
use crate::lens;

pub fn window_main(renderer: Arc<Mutex<FractalRenderer>>) -> impl Widget<FractalData> {
    let render_screen = Align::centered(FractalWidget {
        image_width: 0,
        image_height: 0,
        save_type: 0,
        newton_pos1: (0.0, 0.0),
        newton_pos2: (0.0, 0.0),
        root_pos_start: (0.0, 0.0),
        root_pos_current: (0.0, 0.0),
        cached_image: None,
        needs_buffer_refresh: true,
        show_selecting_box: false,
        renderer_zoom: FloatExtended::new(0.0, 0),
        renderer_rotate: (0.0, 0.0),
    });

    let group_image_size = Flex::column()
        .with_child(Label::new("SIZING").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Flex::column()
                .with_child(create_label_textbox_row("Width:", 75.0)
                    .lens(FractalData::image_width))
                .with_spacer(4.0)
                .with_child(create_label_textbox_row("Height:", 75.0)
                    .lens(FractalData::image_height)), 0.5)
            .with_spacer(4.0)
            .with_flex_child(Flex::column()
                .with_child(Button::new("HALF").on_click(|_ctx, data: &mut FractalData, _env| {
                    data.image_width /= 2;
                    data.image_height /= 2;
                }).expand_width())
                .with_spacer(4.0)
                .with_child(Button::new("DOUBLE").on_click(|_ctx, data: &mut FractalData, _env| {
                    data.image_width *= 2;
                    data.image_height *= 2;
                }).expand_width()), 0.25)
            .with_spacer(4.0)
            .with_flex_child(Flex::column()
                .with_child(Button::new("NATIVE").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(NATIVE_SIZE);
                }).expand_width())
                .with_spacer(4.0)
                .with_child(Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
                    ctx.submit_command(SET_SIZE.with((data.image_width, data.image_height)));
                }).expand_width()), 0.25));

    let group_location = Flex::column()
        .with_child(Label::new("POSITION").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Zoom:").with_text_size(14.0))
            .with_flex_spacer(1.0)
            .with_child(NoUpdateLabel::new(24.0).lens(FractalData::zoom.map(|val| {
                format!("{:>12}", extended_to_string_short(string_to_extended(val)))
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Zoom Factor: ").with_text_size(14.0))
            .with_flex_child(Slider::new()
                .with_range(0.0, 5.64385618977).expand_width()
                .lens(FractalData::zoom_scale_factor.map(
                    |val| val.log2(), 
                    |val, new| *val = 0.1 * (2.0_f64.powf(new) * 10.0).round())), 1.0)
            .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                format!("{:>4.1}", *data)
            }).lens(FractalData::zoom_scale_factor)))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new("ZOOM IN").on_click(|ctx, data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_ZOOM.with(data.zoom_scale_factor));
            }).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("ZOOM OUT").on_click(|ctx, data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_ZOOM.with(1.0 / data.zoom_scale_factor));
            }).expand_width().fix_height(24.0), 1.0))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Iterations:").with_text_size(14.0))
            .with_flex_spacer(1.0)
            .with_child(NoUpdateLabel::new(24.0).lens(FractalData::iteration_limit.map(|val| {
                format!("{:>12}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new("DOUBLE ITERATIONS").on_click(|ctx, data: &mut FractalData, _env| {
                ctx.submit_command(SET_ITERATIONS.with(2 * data.iteration_limit));
            }).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("HALF ITERATIONS").on_click(|ctx, data: &mut FractalData, _env| {
                ctx.submit_command(SET_ITERATIONS.with(data.iteration_limit / 2));
            }).expand_width().fix_height(24.0), 1.0))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Rotation: ").with_text_size(14.0))
            .with_flex_child(Slider::new()
                .with_range(0.0, 72.0).expand_width()
                .lens(FractalData::rotation.map(
                    |val| val / 5.0, 
                    |val, new| *val = new.round() * 5.0)), 1.0)
            .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                format!("{:>5.1}", *data)
            }).lens(FractalData::rotation)))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new("EDIT").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.new_window(WindowDesc::new(window_location()).title(
                    LocalizedString::new("Location"),
                ).window_size((800.0, 400.0)).resizable(true));
            }).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(OPEN_LOCATION);
            }).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(SET_LOCATION);
            }).expand_width().fix_height(24.0), 1.0))
        .with_spacer(8.0)
        .with_child(Label::new("ROOT FINDING").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Root Zoom:").with_text_size(14.0))
            .with_flex_spacer(1.0)
            .with_child(NoUpdateLabel::new(24.0).lens(FractalData::root_zoom.map(|val| {
                let output = if val.len() > 0 {
                    extended_to_string_short(string_to_extended(val))
                } else {
                    String::new()
                };

                format!("{:>12}", output)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Root Period: ").with_text_size(14.0))
            .with_flex_spacer(1.0)
            .with_child(NoUpdateLabel::new(24.0).lens(FractalData::period.map(|val| {
                format!("{:>12}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("Pattern Zoom: ").with_text_size(14.0))
            .with_flex_child(Slider::new()
                .with_range(-1.0,4.0).expand_width()
                .lens(FractalData::root_zoom_factor.map(
                    |val| (1.0 / (1.0 - val)).log2(), 
                    |val, new| *val = 1.0 - (1.0 / 2.0_f64.powf(new.round())))), 1.0)
        .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
            // here need to work out pattern types
            let temp = 1.0 / (1.0 - *data);

            if temp < 1.0 {
                format!("{:>3.1}X", temp)
            } else {
                format!("{:>3.0}X", temp)
            }
        }).lens(FractalData::root_zoom_factor)))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(ProgressBar::new().lens(FractalData::root_progress).expand_width(), 0.5)
            .with_spacer(4.0)
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::root_iteration.map(|val| {
                format!("{:>2}/64", val)
            }, |_, _| {})))
            .with_spacer(4.0)
            .with_spacer(4.0)
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::root_stage.map(|val| {
                let text = match val {
                    1 => "RUNNING",
                    2 => "ERROR",
                    0 => "COMPLETE",
                    _ => "DEFAULT"
                };
    
                format!("{:>8}", text)                
            }, |_, _| {})))
            .with_flex_child(Button::new(|_data: &FractalData, _env: &_| {
                "CANCEL".to_string()
            }).on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(STOP_ROOT_FINDING);
            }).expand_width(), 0.25))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new(|data: &usize, _: &Env| {
                    if *data == 0 {
                        "DRAW BOX".to_string()
                    } else {
                        "CANCEL".to_string()
                    }
                }).on_click(|_ctx, data: &mut usize, _env| {
                    *data = if *data == 0 {
                        1
                    } else {
                        0
                    };
                }).lens(FractalData::mouse_mode).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("CENTRAL OUT").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_PATTERN.with(-1.0));
            }).expand_width().fix_height(24.0), 1.0)
            .with_spacer(4.0)
            .with_flex_child(Button::new("CENTRAL IN").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_PATTERN.with(0.5));
            }).expand_width().fix_height(24.0), 1.0));

    let group_palette = Flex::column()
        .with_child(Flex::row()
            .with_flex_child(Label::new("COLOURING").with_text_size(20.0).expand_width(), 0.5)
            .with_flex_child(Label::new(|data: &FractalData, _env: &_| {
                data.palette_source.clone()
            }).align_right().expand_width(), 0.5))
        .with_spacer(4.0)
        .with_child(Flex::column()
            .with_child(Image::new({
                    let buffer = renderer.lock().data_export.clone();
                    let step = (buffer.lock().palette_interpolated_buffer.len() / 500).max(1);
                    let raw_buffer = buffer.lock().palette_interpolated_buffer.iter().step_by(step).map(|value| {
                        let (r, g, b, _) = value.rgba_u8();
                        vec![r, g, b]
                    }).flatten().collect::<Vec<u8>>();

                    let width = raw_buffer.len() / 3;
                    ImageBuf::from_raw(raw_buffer, ImageFormat::Rgb, width, 1)
                }).interpolation_mode(InterpolationMode::Bilinear)
                .fill_mode(FillStrat::Fill)
                .controller(PaletteUpdateController)
                .fix_height(24.0)
                .expand_width())
            .with_spacer(4.0)
            .with_child(create_label_textbox_row("Span:", 160.0)
                .lens(FractalData::palette_iteration_span))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Offset:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 1.0)
                    .expand_width()
                    .lens(FractalData::palette_offset), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::palette_offset))))
            .with_spacer(4.0)
            .with_child(create_checkbox_row("Cyclic palette").lens(FractalData::palette_cyclic))
            .with_spacer(4.0)
            .with_child(create_label_textbox_row("Stripe Scale:", 160.0)
                .lens(FractalData::stripe_scale))
            .with_spacer(4.0)
            .with_child(create_label_textbox_row("Distance Transition:", 160.0)
                .lens(FractalData::distance_transition))
            .with_spacer(4.0)
            .with_child(create_checkbox_row("Lighting").lens(FractalData::lighting))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Direction:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 360.0)
                    .expand_width()
                    .lens(FractalData::lighting_direction), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_direction)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Azimuth:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 90.0)
                    .expand_width()
                    .lens(FractalData::lighting_azimuth), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_azimuth)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Opacity:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 1.0)
                    .expand_width()
                    .lens(FractalData::lighting_opacity), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_opacity)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Ambient:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 1.0)
                    .expand_width()
                    .lens(FractalData::lighting_ambient), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_ambient)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Diffuse:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 1.0)
                    .expand_width()
                    .lens(FractalData::lighting_diffuse), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_diffuse)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Specular:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 1.0)
                    .expand_width()
                    .lens(FractalData::lighting_specular), 1.0)
                .with_child(Label::<f64>::new(|data: &f64, _env: &_| {
                    format!("{:>.3}", *data)
                }).lens(FractalData::lighting_specular)))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::new("Lighting Shininess:").fix_width(100.0))
                .with_flex_child(Slider::new()
                    .with_range(0.0, 20.0)
                    .expand_width()
                    .lens(FractalData::lighting_shininess.map(|val| *val as f64, |new, val| *new = val as i64)), 1.0)
                .with_child(Label::<i64>::new(|data: &i64, _env: &_| {
                    format!("{:>3}", *data)
                }).lens(FractalData::lighting_shininess)))
            .with_spacer(4.0)
            .with_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(SET_OFFSET_SPAN);
                }).expand_width().fix_height(36.0));

    let group_information = Flex::column()
        .with_child(Flex::row()
        .with_flex_child(Label::new("Skipped:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::min_valid_iterations.map(|val| {
                format!("min. {:>8}", val)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::max_valid_iterations.map(|val| {
                format!("max. {:>8}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("Iterations:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::min_iterations.map(|val| {
                format!("min. {:>8}", val)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::max_iterations.map(|val| {
                format!("max. {:>8}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("Render:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::rendering_stage.map(|val| {
                let text = match val {
                    1 => "Reference",
                    2 | 3 => "Approximation",
                    4 => "Iteration",
                    5 => "Correction",
                    _ => "Complete",
                };
    
                format!("{:>14}", text)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::reference_count.map(|val| {
                format!("{:>8}", format!("Ref:{}", val))
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new(12.0).lens(FractalData::rendering_time.map(|val| {
                let ms = val % 1000;
                let s = val / 1000;
                let m = s / 60;
                let h = m / 60;
    
                format!("{}:{:0>2}:{:0>2}.{:0>3}", h, m % 60, s % 60, ms)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(ProgressBar::new().lens(FractalData::rendering_progress).expand_width(), 0.75)
            .with_spacer(4.0)
            .with_flex_child(Button::new(|data: &FractalData, _env: &_| {
                match data.rendering_stage {
                    0 => {
                        if data.zoom_out_enabled {
                            "CANCEL".to_string()
                        } else {
                            "RESET".to_string()
                        }   
                    },
                    _ => {
                        "CANCEL".to_string()
                    }
                }
            }).on_click(|ctx, data: &mut FractalData, _env| {
                if data.rendering_stage == 0 && !data.zoom_out_enabled {
                    // TODO maybe add a section here that checks if a zoom out sequence is ongoing
                    ctx.submit_command(RESET_RENDERER_FAST);
                } else {
                    // println!("stop called");
                    ctx.submit_command(STOP_RENDERING);
                }
            }).expand_width(), 0.25));

    let group_pixel_information = Flex::column()
        .with_child(Flex::row()
            .with_flex_spacer(0.5)
            .with_child(
                Image::new(ImageBuf::from_raw(vec![0u8; 81 * 3], ImageFormat::Rgb, 9, 9))
                    .interpolation_mode(InterpolationMode::NearestNeighbor)
                    .fill_mode(FillStrat::Contain)
                    .controller(PixelInformationUpdateController)
                    .fix_height(100.0))
            .with_spacer(4.0)
            .with_child(Flex::column()
                .with_child(NoUpdateLabel::new(12.0).lens(FractalData::pixel_pos.map(|val| {
                    format!("{:>13}", format!("({},{})", val[0], val[1]))
                }, |_, _| {})))
                .with_child(Flex::row()
                    .with_child(NoUpdateLabel::new(12.0).lens(FractalData::pixel_iterations.map(|val| {
                        format!("{:>8}", val)
                    }, |_, _| {})))
                    .with_child(NoUpdateLabel::new(12.0).lens(FractalData::pixel_smooth.map(|val| {
                        format!("{:>.4}", val)
                    }, |_, _| {})))))
            .with_flex_spacer(0.5));

    let group_general_information = Flex::column()
        .with_child(Label::new(format!("rust-fractal-gui {}", env!("CARGO_PKG_VERSION"))))
        .with_child(Label::new(format!("{} {} {}", env!("VERGEN_GIT_SHA_SHORT"), env!("VERGEN_GIT_COMMIT_DATE"), env!("VERGEN_GIT_COMMIT_TIME"))))
        .with_child(Label::new(format!("{} {}", env!("VERGEN_RUSTC_SEMVER"), env!("VERGEN_RUSTC_HOST_TRIPLE"))));

    let button_save_advanced_options = Button::new("SAVE & UPDATE").on_click(|ctx, _data, _env| {
        ctx.submit_command(SET_ADVANCED_OPTIONS);
    }).expand_width().fix_height(40.0);

    let group_advanced_options = Flex::column()
        .with_child(create_checkbox_row("Show glitched pixels").lens(FractalData::display_glitches))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Use experimental algorithm").lens(FractalData::experimental))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Jitter pixels").lens(FractalData::jitter))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Automatically adjust iterations").lens(FractalData::auto_adjust_iterations))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Remove image centre").lens(FractalData::remove_centre))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::new("S.A. Order:").fix_width(100.0))
            .with_flex_child(Slider::new()
                .with_range(1.0, 16.0)
                .expand_width()
                .lens(FractalData::order.map(
                    |val| (*val / 4) as f64, 
                    |val, new| *val = 4 * new as i64)), 1.0)
            .with_child(Label::<i64>::new(|data: &i64, _env: &_| {
                format!("{:>3}", *data)
            }).lens(FractalData::order)))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH TOL:", 120.0).lens(FractalData::glitch_tolerance))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH %:", 120.0).lens(FractalData::glitch_percentage))
        .with_child(Flex::row()
            .with_child(Label::new("Data Int.:").fix_width(100.0))
            .with_flex_child(Slider::new()
                .with_range(0.0, 5.0)
                .expand_width()
                .lens(FractalData::iteration_interval.map(
                    |val| (*val as f64).log10(), 
                    |val, new| *val = 10_i64.pow(new as u32))), 1.0)
            .with_child(Label::<i64>::new(|data: &i64, _env: &_| {
                format!("{:>6}", *data)
            }).lens(FractalData::iteration_interval)))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("PROBE SAMPLING:", 100.0).lens(FractalData::probe_sampling))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("JITTER FACTOR:", 100.0).lens(FractalData::jitter_factor));

    let group_extra = Flex::column()
        .with_child(button_save_advanced_options)
        .with_child(group_advanced_options);

    let tabs_menu = Either::new(|data: &FractalData, _env| data.current_tab)
        .add_branch(Flex::column()
            .with_child(group_image_size)
            .with_spacer(8.0)
            .with_child(group_palette)
        )
        .add_branch(group_location)
        .add_branch(group_extra);

    let tabs_selector = Flex::row()
        .with_flex_child(Button::from_label(Label::new("IMAGE").with_text_size(16.0)).on_click(|_ctx, data: &mut FractalData, _env| {
            data.current_tab = 0;
        }).expand_width().fix_height(40.0), 1.0)
        .with_flex_child(Button::from_label(Label::new("LOCATION").with_text_size(16.0)).on_click(|_ctx, data: &mut FractalData, _env| {
            data.current_tab = 1;
        }).expand_width().fix_height(40.0), 1.0)
        .with_flex_child(Button::from_label(Label::new("ADVANCED").with_text_size(16.0)).on_click(|_ctx, data: &mut FractalData, _env| {
            data.current_tab = 2;
        }).expand_width().fix_height(40.0), 1.0);

    let tabs_indicator = Flex::row()
        .with_flex_child(Painter::new(|ctx, data: &usize, env| {
                let bounds = ctx.size().to_rect();
                if *data == 0 {
                    ctx.fill(bounds, &env.get(PRIMARY_DARK));
                } else {
                    ctx.fill(bounds, &env.get(BACKGROUND_DARK));
                }
            }).fix_height(2.0).lens(FractalData::current_tab), 1.0)
        .with_flex_child(Painter::new(|ctx, data: &usize, env| {
                let bounds = ctx.size().to_rect();
                if *data == 1 {
                    ctx.fill(bounds, &env.get(PRIMARY_DARK));
                } else {
                    ctx.fill(bounds, &env.get(BACKGROUND_DARK));
                }
            }).fix_height(2.0).lens(FractalData::current_tab), 1.0)
        .with_flex_child(Painter::new(|ctx, data: &usize, env| {
                let bounds = ctx.size().to_rect();
                if *data == 2 {
                    ctx.fill(bounds, &env.get(PRIMARY_DARK));
                } else {
                    ctx.fill(bounds, &env.get(BACKGROUND_DARK));
                }
            }).fix_height(2.0).lens(FractalData::current_tab), 1.0);

    // TODO have a help and about menu
    let side_menu = Flex::column()
        .with_child(tabs_selector)
        .with_child(tabs_indicator)
        .with_spacer(8.0)
        .with_flex_child(Flex::row()
            .with_flex_spacer(0.05)
            .with_flex_child(Flex::column()
                .with_flex_child(Flex::column()
                    .with_child(tabs_menu)
                    .with_flex_spacer(1.0), 1.0)
                .with_child(group_pixel_information)
                .with_spacer(24.0)
                .with_child(group_information)
                .with_spacer(24.0)
                .with_child(group_general_information)
                .with_spacer(8.0), 0.9)
            .with_flex_spacer(0.05)
            .cross_axis_alignment(CrossAxisAlignment::Start), 1.0);

    Split::columns(render_screen, side_menu).split_point(0.75).draggable(true).solid_bar(true).bar_size(4.0)
}

fn create_label_textbox_row<T: Data + Display + FromStr>(label: &str, width: f64) -> impl Widget<T> where <T as FromStr>::Err: std::error::Error, T: std::fmt::Debug {
    let label = Label::<T>::new(label).with_text_size(14.0);

    let text_box = TextBox::new()
        .with_formatter(ParseFormatter::new())
        .update_data_while_editing(true)
        .expand_width();

    Flex::row()
        .with_child(label.fix_width(width))
        .with_flex_child(text_box, 1.0)
}

fn create_checkbox_row(label: &str) -> impl Widget<bool> {
    let label = Label::<bool>::new(label)
        .expand_width();

    let check_box = Checkbox::new("");

    Flex::row()
        .with_flex_child(label, 1.0)
        .with_spacer(4.0)
        .with_child(check_box)
}

pub fn make_menu(_: Option<WindowId>, _state: &FractalData, _: &Env) -> Menu<FractalData> {
    Menu::empty()
        .entry(Menu::new(LocalizedString::new("File"))
            .entry(MenuItem::new(LocalizedString::new("Open")).command(OPEN_LOCATION).hotkey(SysMods::Cmd, "o"))
            .entry(MenuItem::new(LocalizedString::new("Save Location")).command(SAVE_LOCATION))
            .entry(MenuItem::new(LocalizedString::new("Save Image")).command(SAVE_IMAGE).hotkey(SysMods::Cmd, "s"))
            .entry(MenuItem::new(LocalizedString::new("Save Configuration")).command(SAVE_ALL))
            .entry(MenuItem::new(LocalizedString::new("Zoom Out Default")).command(ZOOM_OUT))
            .entry(MenuItem::new(LocalizedString::new("Zoom Out Removed")).command(ZOOM_OUT_OPTIMISED))
            .entry(MenuItem::new(LocalizedString::new("Exit")).command(CLOSE_ALL_WINDOWS)))
        .entry(Menu::new(LocalizedString::new("common-menu-edit-menu"))
            .entry(MenuItem::new(LocalizedString::new("Reset")).command(RESET_DEFAULT_LOCATION).hotkey(SysMods::Cmd, "r"))
            .entry(druid::platform_menus::common::cut())
            .entry(druid::platform_menus::common::copy())
            .entry(druid::platform_menus::common::paste()))
        .entry(Menu::new(LocalizedString::new("Colouring"))
            .entry(MenuItem::new(LocalizedString::new("Smooth Iteration")).command(SET_COLORING_METHOD.with(ColoringType::SmoothIteration)))
            .entry(MenuItem::new(LocalizedString::new("Step Iteration")).command(SET_COLORING_METHOD.with(ColoringType::StepIteration)))
            .entry(MenuItem::new(LocalizedString::new("Distance")).command(SET_COLORING_METHOD.with(ColoringType::Distance)))
            .entry(MenuItem::new(LocalizedString::new("Stripe")).command(SET_COLORING_METHOD.with(ColoringType::Stripe)))
            .entry(MenuItem::new(LocalizedString::new("Distance Stripe")).command(SET_COLORING_METHOD.with(ColoringType::DistanceStripe)))
    )
}

pub fn window_location() -> impl Widget<FractalData> {
    Flex::row()
        .with_flex_spacer(0.05)
        .with_flex_child(Flex::column()
            .with_spacer(8.0)
            .with_child(Label::new("Real:").with_text_size(14.0))
            .with_spacer(8.0)
            .with_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::RealLens))
            .with_spacer(8.0)
            .with_child(Label::<FractalData>::new("Imag:").with_text_size(14.0))
            .with_spacer(8.0)
            .with_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::ImagLens))
            .with_spacer(8.0)
            .with_child(Label::new("Zoom:").with_text_size(14.0))
            .with_spacer(8.0)
            .with_child(TextBox::new().with_text_size(10.0).expand_width().lens(lens::ZoomLens))
            .with_spacer(8.0)
            .with_child(Label::new("Iterations:").with_text_size(14.0))
            .with_spacer(8.0)
            .with_child(TextBox::new()
                .with_text_size(10.0)
                .with_formatter(ParseFormatter::new())
                .update_data_while_editing(true)
                .expand_width()
                .lens(FractalData::iteration_limit))
            .with_spacer(8.0)
            .with_child(Flex::row()
                .with_flex_spacer(0.25)
                .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(Command::new(SET_LOCATION, (), Target::Global));
                    ctx.submit_command(CLOSE_WINDOW);
                }).expand_width().fix_height(32.0), 0.25)
                .with_spacer(4.0)
                .with_flex_child(Button::new("CANCEL").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(Command::new(REVERT_LOCATION, (), Target::Global));
                    ctx.submit_command(CLOSE_WINDOW);
                }).expand_width().fix_height(32.0), 0.25)
                .with_flex_spacer(0.25))
            .with_spacer(8.0)
            .cross_axis_alignment(CrossAxisAlignment::Start), 0.9)
        .with_flex_spacer(0.05)
        .scroll()
        .vertical()
}