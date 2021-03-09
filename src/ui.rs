use std::{fmt::Display, str::FromStr};
use druid::widget::{Align, Button, Checkbox, FillStrat, Flex, Image, Label, ProgressBar, Slider, Split, TextBox, WidgetExt, CrossAxisAlignment};
use druid::{Widget, ImageBuf, Data, LensExt};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::text::format::ParseFormatter;
use parking_lot::Mutex;
use std::sync::Arc;
use rust_fractal::renderer::FractalRenderer;

use crate::{FractalData, FractalWidget, custom::Either, custom::PaletteUpdateController, custom::{RenderTimer, SkippedLabel, IterationsLabel}};
use crate::commands::*;
use crate::lens;

pub fn ui_builder(renderer: Arc<Mutex<FractalRenderer>>) -> impl Widget<FractalData> {
    let render_screen = Align::centered(FractalWidget {
        buffer: Vec::new(),
        image_width: 0,
        image_height: 0,
        save_type: 0,
    });

    let group_image_size = Flex::column()
        .with_child(Label::<FractalData>::new("RESOLUTION").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_flex_child(Flex::column()
                .with_child(create_label_textbox_row("WIDTH:", 80.0)
                    .lens(FractalData::image_width))
                .with_spacer(4.0)
                .with_child(create_label_textbox_row("HEIGHT:", 80.0)
                    .lens(FractalData::image_height)), 0.75)
            .with_spacer(4.0)
            .with_flex_child(Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
                ctx.submit_command(SET_SIZE.with((data.image_width, data.image_height)));
            }).expand_width().fix_height(36.0), 0.25))
        .with_spacer(6.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new("HALF").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_SIZE.with(0.5));
            }).expand_width(), 1.0)
            .with_spacer(2.0)
            .with_flex_child(Button::new("DOUBLE").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(MULTIPLY_SIZE.with(2.0));
            }).expand_width(), 1.0)
            .with_spacer(2.0)
            .with_flex_child(Button::new("NATIVE").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(NATIVE_SIZE);
            }).expand_width(), 1.0));

    let group_location = Flex::column()
        .with_child(Flex::row()
            .with_flex_child(Label::<FractalData>::new("LOCATION").with_text_size(20.0).expand_width(), 0.5)
            .with_flex_child(Label::new(|data: &FractalData, _env: &_| {
                data.location_source.clone()
            }).expand_width(), 0.5))
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
                }).expand_width(), 0.15)))
        .with_spacer(6.0)
        .with_child(Flex::row()
            .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(SET_LOCATION);
            }).expand_width(), 1.0)
            .with_spacer(2.0)
            .with_flex_child(Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(OPEN_LOCATION);
            }).expand_width(), 1.0)
            .with_spacer(2.0)
            .with_flex_child(Button::new("SAVE").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(SAVE_LOCATION);
            }).expand_width(), 1.0));

    let group_palette = Flex::column()
        .with_child(Label::<FractalData>::new("COLOURING").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::column()
            .with_child(Flex::row()
                .with_child(Label::<FractalData>::new("METHOD:").with_text_size(14.0).fix_width(90.0))
                .with_flex_child(Label::new(|data: &FractalData, _env: &_| {
                    if data.settings.lock().get_bool("analytic_derivative").unwrap() {
                        "distance".to_string()
                    } else {
                        "iteration".to_string()
                    }
                }).with_text_size(12.0).expand_width(), 0.6)
                .with_spacer(2.0)
                .with_flex_child(Button::new("TOGGLE").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(TOGGLE_DERIVATIVE);
                }).expand_width(), 0.4))
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_child(Label::<FractalData>::new("PALETTE:").with_text_size(14.0).fix_width(90.0))
                .with_flex_child(Label::new(|data: &FractalData, _env: &_| {
                    data.palette_source.clone()
                }).with_text_size(8.0).expand_width(), 0.6)
                .with_spacer(2.0)
                .with_flex_child(Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(OPEN_LOCATION);
                }).expand_width(), 0.4))
            .with_spacer(4.0)
            .with_child(
                Image::new(ImageBuf::from_raw(renderer.lock().data_export.lock().palette_generator.colors(100).iter().map(|value| {
                    let (r, g, b, _) = value.rgba_u8();
                    vec![r, g, b]
                }).flatten().collect::<Vec<u8>>(), ImageFormat::Rgb, 100, 1))
                    .interpolation_mode(InterpolationMode::Bilinear)
                    .fill_mode(FillStrat::Fill)
                    .controller(PaletteUpdateController)
                    .fix_height(12.0)
                    .expand_width())
            .with_spacer(4.0)
            .with_child(Flex::row()
                .with_flex_child(Flex::column()
                    .with_child(create_label_textbox_row("DIVISION:", 90.0)
                        .lens(FractalData::iteration_division))
                    .with_spacer(4.0)
                    .with_child(create_label_textbox_row("OFFSET:", 90.0)
                        .lens(FractalData::iteration_offset)), 0.7)
                .with_spacer(4.0)
                .with_flex_child(Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
                    ctx.submit_command(SET_OFFSET_DIVISION);
                }).expand_width().fix_height(36.0), 0.3)));

    let group_options = Flex::column()
        .with_child(Label::<FractalData>::new("OPTIONS").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("EXPORT:").with_text_size(14.0).fix_width(90.0))
            .with_flex_child(Button::new("IMAGE").on_click(|ctx, _data: &mut FractalData, _env| {
                ctx.submit_command(SAVE_IMAGE);
            }).expand_width(), 1.0)
        .with_spacer(2.0)
        .with_flex_child(Button::new("SETTINGS").on_click(|ctx, _data: &mut FractalData, _env| {
            ctx.submit_command(SAVE_ALL);
        }).expand_width(), 1.0));

    let group_information = Flex::column()
        .with_child(Label::<FractalData>::new("INFORMATION").with_text_size(20.0).expand_width())
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("SKIPPED:").with_text_size(14.0).fix_width(50.0))
            .with_flex_child(SkippedLabel::new().align_right(), 1.0))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("ITERATIONS:").with_text_size(14.0).fix_width(50.0))
            .with_flex_child(IterationsLabel::new().align_right(), 1.0))
        .with_spacer(4.0)
        .with_child(Flex::row()
            .with_child(Label::<FractalData>::new("RENDER:").with_text_size(14.0).fix_width(50.0))
            .with_flex_child(RenderTimer::new().align_right(), 1.0))
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

    let button_toggle_menu = Button::new("ADVANCED OPTIONS").on_click(|_ctx, data: &mut bool, _env| {
        *data = true;
    }).lens(FractalData::show_settings).expand_width();

    // TODO have a help and about menu
    let side_menu = Flex::row()
        .with_flex_spacer(0.05)
        .with_flex_child(Flex::column()
            .with_spacer(8.0)
            .with_child(group_image_size)
            .with_spacer(8.0)
            .with_child(group_location)
            .with_spacer(8.0)
            .with_child(group_palette)
            .with_spacer(8.0)
            .with_child(group_options)
            .with_spacer(8.0)
            .with_child(group_information)
            .with_spacer(4.0)
            .with_child(button_start_zoom_out)
            .with_spacer(4.0)
            .with_child(button_start_zoom_out_optimised)
            .with_spacer(4.0)
            .with_child(button_toggle_menu), 0.9)
        .with_flex_spacer(0.05)
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let label_advanced_options = Label::<FractalData>::new("ADVANCED OPTIONS").with_text_size(20.0).expand_width();
    let button_save_advanced_options = Button::new("SAVE & UPDATE").on_click(|ctx, data: &mut bool, _env| {
        *data = false;
        ctx.submit_command(SET_ADVANCED_OPTIONS);
    }).lens(FractalData::show_settings).expand_width().fix_height(40.0);

    let row_advanced_options = Flex::row()
        .with_flex_child(label_advanced_options, 0.8)
        .with_spacer(8.0)
        .with_flex_child(button_save_advanced_options, 0.2);

    let section_checkboxes = Flex::column()
        .with_child(create_checkbox_row("Show glitched pixels").lens(FractalData::display_glitches))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Use experimental algorithm").lens(FractalData::experimental))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Jitter pixels").lens(FractalData::jitter))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Automatically adjust iterations").lens(FractalData::auto_adjust_iterations))
        .with_spacer(4.0)
        .with_child(create_checkbox_row("Remove image centre").lens(FractalData::remove_centre));

    let section_values = Flex::column()
        .with_child(create_label_slider_row("SERIES APPROXIMATION ORDER:", 280.0, 4.0, 128.0)
            .lens(FractalData::order.map(|val| *val as f64, |val, new| *val = new as i64)))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH TOLERANCE:", 280.0)
            .lens(FractalData::glitch_tolerance))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("GLITCH PERCENTAGE:", 280.0)
            .lens(FractalData::glitch_percentage))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("DATA STORAGE INTERVAL:", 280.0)
            .lens(FractalData::iteration_interval))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("PROBE SAMPLING:", 280.0)
            .lens(FractalData::probe_sampling))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("JITTER FACTOR:", 280.0)
            .lens(FractalData::jitter_factor));

    let section_advanced_options = Flex::row()
        .with_flex_child(section_checkboxes, 0.5)
        .with_spacer(32.0)
        .with_flex_child(section_values, 0.5);

    let row_real = Flex::row()
        .with_child(Label::<FractalData>::new("REAL:").with_text_size(14.0).fix_width(60.0))
        .with_flex_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::RealLens), 1.0);

    let row_imag = Flex::row()
        .with_child(Label::<FractalData>::new("IMAG:").with_text_size(14.0).fix_width(60.0))
        .with_flex_child(TextBox::multiline().with_text_size(10.0).expand_width().lens(lens::ImagLens), 1.0);

    let column_advanced_options = Flex::column()
        .with_spacer(8.0)
        .with_child(row_advanced_options)
        .with_spacer(4.0)
        .with_child(section_advanced_options)
        .with_spacer(32.0)
        .with_child(row_real)
        .with_spacer(4.0)
        .with_child(row_imag);
    
    let advanced_options_menu = Flex::row()
        .with_flex_spacer(0.05)
        .with_flex_child(column_advanced_options, 0.9)
        .with_flex_spacer(0.05)
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let either_main_screen = Either::new(|data: &FractalData, _env| {
            data.show_settings
        }, advanced_options_menu, render_screen);

    Split::columns(either_main_screen, side_menu).split_point(0.75).draggable(true).solid_bar(true).bar_size(4.0)
}

fn create_label_textbox_row<T: Data + Display + FromStr>(label: &str, width: f64) -> impl Widget<T> where <T as FromStr>::Err: std::error::Error {
    let label = Label::<T>::new(label).with_text_size(14.0);

    let text_box = TextBox::new()
        .with_formatter(ParseFormatter::new())
        .update_data_while_editing(true)
        .expand_width();

    Flex::row()
        .with_child(label.fix_width(width))
        .with_flex_child(text_box, 1.0)
}

fn create_label_slider_row(label: &str, width: f64, min: f64, max: f64) -> impl Widget<f64> {
    let label = Label::<f64>::new(label).with_text_size(14.0);

    let slider = Slider::new()
        .with_range(min, max)
        .expand_width();

    let value = Label::<f64>::new(|data: &f64, _env: &_| {
        format!("{:>3}", *data as i64)
    });

    Flex::row()
        .with_child(label.fix_width(width))
        .with_flex_child(slider, 1.0)
        .with_child(value)
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