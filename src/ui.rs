use std::{fmt::Display, str::FromStr};
use druid::widget::{Align, Button, Checkbox, FillStrat, Flex, Image, Label, ProgressBar, Slider, Split, TextBox, WidgetExt, CrossAxisAlignment};
use druid::{Widget, ImageBuf, Data, LensExt, Menu, LocalizedString, MenuItem, SysMods, Env, WindowId};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::text::ParseFormatter;
use druid::commands::CLOSE_ALL_WINDOWS;

use parking_lot::Mutex;
use std::sync::Arc;
use rust_fractal::renderer::FractalRenderer;

use crate::{FractalData, FractalWidget, ColoringType};
use crate::custom::*;


use crate::commands::*;
use crate::lens;

pub fn ui_builder(renderer: Arc<Mutex<FractalRenderer>>) -> impl Widget<FractalData> {
    let render_screen = Align::centered(FractalWidget {
        buffer: Vec::new(),
        image_width: 0,
        image_height: 0,
        save_type: 0,
        newton_pos1: (0.0, 0.0),
        newton_pos2: (0.0, 0.0),
    });

    let group_image_size = Flex::column()
        .with_child(Flex::row()
            .with_flex_child(Flex::column()
                .with_child(create_label_textbox_row("WIDTH:", 75.0)
                    .lens(FractalData::image_width))
                .with_spacer(4.0)
                .with_child(create_label_textbox_row("HEIGHT:", 75.0)
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
        .with_child(Flex::row()
            .with_flex_child(Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(OPEN_LOCATION);
            }).expand_width(), 0.4)
            .with_spacer(4.0)
            .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(SET_LOCATION);
            }).expand_width(), 0.4))
        .with_spacer(4.0)
        .with_child(Flex::column()
            .with_child(Flex::row()
                .with_child(Label::<FractalData>::new("ZOOM:").with_text_size(14.0).fix_width(60.0))
                .with_flex_child(TextBox::new()
                    .with_formatter(ParseFormatter::new()).update_data_while_editing(true).expand_width().lens(FractalData::zoom_mantissa), 0.5)
                .with_spacer(2.0)
                .with_child(Label::<FractalData>::new("E").with_text_size(14.0))
                .with_spacer(2.0)
                .with_flex_child(TextBox::new()
                    .with_formatter(ParseFormatter::new()).update_data_while_editing(true).expand_width().lens(FractalData::zoom_exponent), 0.2)
                .with_spacer(4.0)
                .with_flex_child(Button::new("+").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(MULTIPLY_ZOOM.with(2.0));
                }).expand_width(), 0.15)
                .with_spacer(2.0)
                .with_flex_child(Button::new("-").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(MULTIPLY_ZOOM.with(0.5));
                }).expand_width(), 0.15))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_flex_child(create_label_textbox_row("ITER:", 60.0).lens(FractalData::maximum_iterations), 0.7)
                .with_spacer(4.0)
                .with_flex_child(Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
                    ctx.submit_command(SET_ITERATIONS.with(2 * data.maximum_iterations));
                }).expand_width(), 0.15)
                .with_spacer(2.0)
                .with_flex_child(Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
                    ctx.submit_command(SET_ITERATIONS.with(data.maximum_iterations / 2));
                }).expand_width(), 0.15))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_flex_child(create_label_textbox_row("ROTN:", 60.0).lens(FractalData::rotation), 0.7)
                .with_spacer(4.0)
                .with_flex_child(Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
                    ctx.submit_command(SET_ROTATION.with(data.rotation - 15.0));
                }).expand_width(), 0.15)
                .with_spacer(2.0)
                .with_flex_child(Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
                    ctx.submit_command(SET_ROTATION.with(data.rotation + 15.0));
                }).expand_width(), 0.15))
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("REAL:").with_text_size(14.0).fix_width(60.0))
            .with_flex_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::RealLens), 1.0))
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("IMAG:").with_text_size(14.0).fix_width(60.0))
            .with_flex_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::ImagLens), 1.0)));

    let group_palette = Flex::column()
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("COLOURING").with_text_size(20.0).expand_width(), 0.5)
            .with_flex_child(Label::new(|data: &FractalData, _env: &_| {
                data.palette_source.clone()
            }).align_right().expand_width(), 0.5))
        .with_spacer(4.0)
        .with_child(Flex::column()
            .with_child(
                Image::new(ImageBuf::from_raw(renderer.lock().data_export.lock().palette_generator.colors(100).iter().map(|value| {
                    let (r, g, b, _) = value.rgba_u8();
                    vec![r, g, b]
                }).flatten().collect::<Vec<u8>>(), ImageFormat::Rgb, 100, 1))
                    .interpolation_mode(InterpolationMode::Bilinear)
                    .fill_mode(FillStrat::Fill)
                    .controller(PaletteUpdateController)
                    .fix_height(24.0)
                    .expand_width())
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_flex_child(Flex::column()
                    .with_child(create_label_textbox_row("SPAN:", 90.0)
                        .lens(FractalData::iteration_span))
                    .with_spacer(4.0)
                    .with_child(create_label_textbox_row("OFFSET:", 90.0)
                        .lens(FractalData::iteration_offset)), 0.7)
                .with_spacer(4.0)
                .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(SET_OFFSET_SPAN);
                }).expand_width().fix_height(36.0), 0.3)));

    let group_information = Flex::column()
        .with_child(Flex::row()
        .with_flex_child(Label::<FractalData>::new("SKIPPED:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new().lens(FractalData::min_valid_iterations.map(|val| {
                format!("min. {:>8}", val)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new().lens(FractalData::max_valid_iterations.map(|val| {
                format!("max. {:>8}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("ITERATIONS:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new().lens(FractalData::min_iterations.map(|val| {
                format!("min. {:>8}", val)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new().lens(FractalData::max_iterations.map(|val| {
                format!("max. {:>8}", val)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("RENDER:").with_text_size(14.0).expand_width(), 1.0)
            .with_child(NoUpdateLabel::new().lens(FractalData::stage.map(|val| {
                let text = match val {
                    1 => "REFERENCE",
                    2 | 3 => "APPROXIMATION",
                    4 => "ITERATION",
                    5 => "CORRECTION",
                    0 => "COMPLETE",
                    _ => "DEFAULT"
                };
    
                format!("{:>14}", text)
            }, |_, _| {})))
            .with_child(NoUpdateLabel::new().lens(FractalData::time.map(|val| {
                let ms = val % 1000;
                let s = val / 1000;
                let m = s / 60;
                let h = m / 60;
    
                format!("{}:{:0>2}:{:0>2}:{:0>3}", h, m % 60, s % 60, ms)
            }, |_, _| {}))))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(ProgressBar::new().lens(FractalData::progress).expand_width(), 0.75)
            .with_spacer(4.0)
            .with_flex_child(Button::new(|data: &FractalData, _env: &_| {
                match data.stage {
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
                if data.stage == 0 && !data.zoom_out_enabled {
                    // TODO maybe add a section here that checks if a zoom out sequence is ongoing
                    ctx.submit_command(RESET_RENDERER_FAST);
                } else {
                    // println!("stop called");
                    ctx.submit_command(STOP_RENDERING);
                }
            }).expand_width(), 0.25));

    let button_start_zoom_out = Button::new("START ZOOM OUT").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(ZOOM_OUT);
    }).expand_width();

    let button_start_zoom_out_optimised = Button::new("START ZOOM OUT OPTIMISED").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(ZOOM_OUT_OPTIMISED);
    }).expand_width();

    let button_toggle_mouse_mode = Button::new("MOUSE MODE").on_click(|_ctx, data: &mut usize, _env| {
        *data = if *data == 0 {
            1
        } else {
            0
        };
    }).lens(FractalData::mouse_mode).expand_width();

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
                .with_child(NoUpdateLabel::new().lens(FractalData::pixel_pos.map(|val| {
                    format!("{:>13}", format!("({},{})", val[0], val[1]))
                }, |_, _| {})))
                .with_child(Flex::row()
                    .with_child(NoUpdateLabel::new().lens(FractalData::pixel_iterations.map(|val| {
                        format!("{:>8}", val)
                    }, |_, _| {})))
                    .with_child(NoUpdateLabel::new().lens(FractalData::pixel_smooth.map(|val| {
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
            }).lens(FractalData::order))
        )
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH TOL:", 120.0)
            .lens(FractalData::glitch_tolerance))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH %:", 120.0)
            .lens(FractalData::glitch_percentage))
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
            }).lens(FractalData::iteration_interval))
        )
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("PROBE SAMPLING:", 100.0)
            .lens(FractalData::probe_sampling))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("JITTER FACTOR:", 100.0)
            .lens(FractalData::jitter_factor));
    
    let group_extra = Flex::column()
        .with_child(button_start_zoom_out)
        .with_child(button_start_zoom_out_optimised)
        .with_child(button_toggle_mouse_mode)
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

    // TODO have a help and about menu
    // TODO have an additional window for the submission of the exact location
    let side_menu = Flex::column()
        .with_child(tabs_selector)
        .with_spacer(4.0)
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
    )
}