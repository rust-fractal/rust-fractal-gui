use std::{fmt::Display, str::FromStr};

use druid::widget::{Align, Button, Checkbox, FillStrat, Flex, Image, Label, LensWrap, ProgressBar, Slider, Split, TextBox, WidgetExt};
use druid::{Widget, ImageBuf, Data, LensExt};
use druid::piet::{ImageFormat, InterpolationMode};
use druid::text::format::ParseFormatter;

use config::{Config, File};

use crate::{FractalData, FractalWidget, custom::Either, custom::PaletteUpdateController, custom::{RenderTimer, SkippedLabel, IterationsLabel}};

use crate::commands::*;

use crate::lens;

pub fn ui_builder() -> impl Widget<FractalData> {
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    let render_screen = FractalWidget {
        buffer: Vec::new(),
        image_width: 0,
        image_height: 0,
        save_type: 0,
    };

    let render_screen = Align::centered(render_screen);

    let resolution_title = Label::<FractalData>::new("RESOLUTION").with_text_size(20.0);

    let row_1 = resolution_title.expand_width();


    let button_set_image_size = Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(SET_SIZE.with((data.image_width, data.image_height)));
    }).expand_width().fix_height(36.0);

    let image_size_column = Flex::column()
        .with_child(create_label_textbox_row("WIDTH:", 80.0)
            .lens(FractalData::image_width))
        .with_spacer(4.0)
        .with_child(create_label_textbox_row("HEIGHT:", 80.0)
            .lens(FractalData::image_height));

    let row_2 = Flex::row()
        .with_flex_child(image_size_column, 0.75)
        .with_spacer(4.0)
        .with_flex_child(button_set_image_size, 0.25);


    let button_half = Button::new("HALF").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(MULTIPLY_SIZE.with(0.5));
    }).expand_width();

    let button_double = Button::new("DOUBLE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(MULTIPLY_SIZE.with(2.0));
    }).expand_width();

    let button_native = Button::new("NATIVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(NATIVE_SIZE);
    }).expand_width();

    let row_3 = Flex::row()
        .with_flex_child(button_half, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_double, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_native, 1.0);


    let file_location = Label::new(|data: &FractalData, _env: &_| {
        data.location_source.clone()
    }).expand_width();

    let row_4 = Flex::row()
        .with_flex_child(Label::<FractalData>::new("LOCATION").with_text_size(20.0).expand_width(), 0.5)
        .with_flex_child(file_location, 0.5);

    let zoom_mantissa = TextBox::new()
        .with_formatter(ParseFormatter::new())
        .update_data_while_editing(true)
        .expand_width()
        .lens(FractalData::zoom_mantissa);

    let zoom_exponent = TextBox::new()
        .with_formatter(ParseFormatter::new())
        .update_data_while_editing(true)
        .expand_width()
        .lens(FractalData::zoom_exponent);

    let button_zoom_in = Button::new("+").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(MULTIPLY_ZOOM.with(2.0));
    }).expand_width();

    let button_zoom_out = Button::new("-").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(MULTIPLY_ZOOM.with(0.5));
    }).expand_width();

    let zoom_row = Flex::row()
        .with_child(Label::<FractalData>::new("ZOOM:").with_text_size(14.0).fix_width(60.0))
        .with_flex_child(zoom_mantissa, 0.4)
        .with_spacer(2.0)
        .with_child(Label::<FractalData>::new("E").with_text_size(14.0))
        .with_spacer(2.0)
        .with_flex_child(zoom_exponent, 0.2)
        .with_spacer(4.0)
        .with_flex_child(button_zoom_in, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_zoom_out, 0.15);

    let button_increase_iterations = Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(SET_ITERATIONS.with(2 * data.maximum_iterations));
    }).expand_width();

    let button_decrease_iterations = Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(SET_ITERATIONS.with(data.maximum_iterations / 2));
    }).expand_width();

    let iterations_section = create_label_textbox_row("ITER:", 60.0)
        .lens(FractalData::max_iterations);

    let iterations_row = Flex::row()
        .with_flex_child(iterations_section, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_increase_iterations, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_decrease_iterations, 0.15);

    let button_increase_rotation = Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(SET_ROTATION.with(data.rotation - 15.0));
    }).expand_width();

    let button_decrease_rotation = Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(SET_ROTATION.with(data.rotation + 15.0));
    }).expand_width();

    let rotation_section = create_label_textbox_row("ROTN:", 60.0)
        .lens(FractalData::rotation);

    let rotation_row = Flex::row()
        .with_flex_child(rotation_section, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_increase_rotation, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_decrease_rotation, 0.15);


    let row_5 = Flex::column()
        .with_child(zoom_row)
        .with_spacer(4.0)
        .with_child(iterations_row)
        .with_spacer(4.0)
        .with_child(rotation_row);

    let button_set_location = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(SET_LOCATION);
    }).expand_width();

    let button_load_location = Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(OPEN_LOCATION);
    }).expand_width();

    let button_save_location = Button::new("SAVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(SAVE_LOCATION);
    }).expand_width();

    let row_6 = Flex::row()
        .with_flex_child(button_set_location, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_load_location, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_save_location, 1.0);

    let mut colouring_title = Label::<FractalData>::new("COLOURING");
    colouring_title.set_text_size(20.0);

    let row_7 = Flex::row()
        .with_flex_child(colouring_title.expand_width(), 1.0);

    let mut colouring_method_label = Label::<FractalData>::new("METHOD:");
    let mut palette_label = Label::<FractalData>::new("PALETTE:");

    colouring_method_label.set_text_size(14.0);
    palette_label.set_text_size(14.0);

    let mut colouring = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock();

        if settings.get_bool("analytic_derivative").unwrap() {
            "distance".to_string()
        } else {
            "iteration".to_string()
        }
    });

    let mut palette = Label::new(|data: &FractalData, _env: &_| {
        data.palette_source.clone()
    });

    colouring.set_text_size(12.0);
    palette.set_text_size(8.0);

    let button_set_method = Button::new("TOGGLE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(TOGGLE_DERIVATIVE);
    }).expand_width();

    let colouring_method_row = Flex::row()
        .with_child(colouring_method_label.fix_width(90.0))
        .with_flex_child(colouring.expand_width(), 0.6)
        .with_spacer(2.0)
        .with_flex_child(button_set_method, 0.4);

    let button_set_palette = Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(OPEN_LOCATION);
    }).expand_width();
    
    let palette_row = Flex::row()
        .with_child(palette_label.fix_width(90.0))
        .with_flex_child(palette.expand_width(), 0.6)
        .with_spacer(2.0)
        .with_flex_child(button_set_palette, 0.4);

    let iteration_division_section = create_label_textbox_row("DIVISION:", 90.0)
        .lens(FractalData::iteration_division);

    let iteration_offset_section = create_label_textbox_row("OFFSET:", 90.0)
        .lens(FractalData::iteration_offset);

    let set_iteration_offset = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(SET_OFFSET_DIVISION);
    }).expand_width().fix_height(36.0);

    let offset_division_column = Flex::column()
        .with_child(iteration_division_section)
        .with_spacer(4.0)
        .with_child(iteration_offset_section);

    let offset_division_row = Flex::row()
        .with_flex_child(offset_division_column, 0.7)
        .with_spacer(4.0)
        .with_flex_child(set_iteration_offset, 0.3);

    let raw_buffer = settings.get_array("palette").unwrap().chunks(3).map(|value| {
        Vec::from([value[2].clone().into_int().unwrap() as u8, value[1].clone().into_int().unwrap() as u8, value[0].clone().into_int().unwrap() as u8])
    }).flatten().collect::<Vec<u8>>();

    let test = ImageBuf::from_raw(raw_buffer.clone(), ImageFormat::Rgb, raw_buffer.len() / 3, 1);

    let test_image = Image::new(test)
        .interpolation_mode(InterpolationMode::Bilinear)
        .fill_mode(FillStrat::Fill)
        .controller(PaletteUpdateController)
        .fix_height(12.0)
        .expand_width();

    let row_8 = Flex::column()
        .with_child(colouring_method_row)
        .with_spacer(4.0)
        .with_child(palette_row)
        .with_spacer(4.0)
        .with_child(test_image)
        .with_spacer(4.0)
        .with_child(offset_division_row);

    let mut options_title = Label::<FractalData>::new("OPTIONS");
    options_title.set_text_size(20.0);

    let row_9 = Flex::row()
        .with_flex_child(options_title.expand_width(), 1.0);

    let mut export_label = Label::<FractalData>::new("EXPORT:");

    export_label.set_text_size(14.0);

    let button_save_image = Button::new("IMAGE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(SAVE_IMAGE);
    }).expand_width();

    let button_refresh_full = Button::new("SETTINGS").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(SAVE_ALL);
    }).expand_width();

    let row_11 = Flex::row()
        .with_child(export_label.fix_width(90.0))
        .with_flex_child(button_save_image, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_refresh_full, 1.0);

    let mut information_title = Label::<FractalData>::new("INFORMATION");
    information_title.set_text_size(20.0);

    let row_12 = Flex::row()
        .with_flex_child(information_title.expand_width(), 1.0);

    let mut skipped_label = Label::<FractalData>::new("SKIPPED:");
    let mut iterations_label = Label::<FractalData>::new("ITERATIONS:");
    let mut render_time_label = Label::<FractalData>::new("RENDER:");

    skipped_label.set_text_size(14.0);
    iterations_label.set_text_size(14.0);
    render_time_label.set_text_size(14.0);

    let skipped = SkippedLabel::new().align_right();
    let iterations = IterationsLabel::new().align_right();
    let render_timer = RenderTimer::new().align_right();

    let row_13 = Flex::row()
        .with_child(skipped_label.fix_width(50.0))
        .with_flex_child(skipped, 1.0);

    let row_14 = Flex::row()
        .with_child(iterations_label.fix_width(50.0))
        .with_flex_child(iterations, 1.0);

    let row_15 = Flex::row()
        .with_child(render_time_label.fix_width(50.0))
        .with_flex_child(render_timer, 1.0);

    let render_progress = ProgressBar::new().lens(FractalData::progress).expand_width();

    let button_toggle_state = Button::new(|data: &FractalData, _env: &_| {
        let text = match data.stage {
            0 => {
                if data.zoom_out_enabled {
                    "CANCEL"
                } else {
                    "RESET"
                }   
            },
            _ => {
                "CANCEL"
            }
        };

        text.to_string()
    }).on_click(|ctx, data: &mut FractalData, _env| {
        if data.stage == 0 && !data.zoom_out_enabled {
            // TODO maybe add a section here that checks if a zoom out sequence is ongoing
            ctx.submit_command(RESET_RENDERER_FAST);
        } else {
            // println!("stop called");
            ctx.submit_command(STOP_RENDERING);
        }
    }).expand_width();

    let button_start_zoom_out = Button::new("START ZOOM OUT").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(ZOOM_OUT);
    }).expand_width();

    let button_start_zoom_out_optimised = Button::new("START ZOOM OUT OPTIMISED").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(ZOOM_OUT_OPTIMISED);
    }).expand_width();

    let button_toggle_menu = Button::new("ADVANCED OPTIONS").on_click(|_ctx, data: &mut bool, _env| {
        *data = true;
    }).lens(FractalData::show_settings).expand_width();

    let row_16 = Flex::row()
        .with_flex_child(render_progress, 0.75)
        .with_spacer(4.0)
        .with_flex_child(button_toggle_state, 0.25);

    // TODO have a help and about menu

    let mut columns = Flex::<FractalData>::column()
        .with_spacer(8.0)
        .with_child(row_1)
        .with_spacer(4.0)
        .with_child(row_2)
        .with_spacer(6.0)
        .with_child(row_3)
        .with_spacer(8.0)
        .with_child(row_4)
        .with_spacer(4.0)
        .with_child(row_5)
        .with_spacer(6.0)
        .with_child(row_6)
        .with_spacer(8.0)
        .with_child(row_7)
        .with_spacer(4.0)
        .with_child(row_8)
        .with_spacer(8.0)
        .with_child(row_9)
        .with_spacer(4.0)
        .with_child(row_11)
        .with_spacer(8.0)
        .with_child(row_12)
        .with_spacer(4.0)
        .with_child(row_13)
        .with_spacer(4.0)
        .with_child(row_14)
        .with_spacer(4.0)
        .with_child(row_15)
        .with_spacer(4.0)
        .with_child(row_16)
        .with_spacer(4.0)
        .with_child(button_start_zoom_out)
        .with_spacer(4.0)
        .with_child(button_start_zoom_out_optimised)
        .with_spacer(4.0)
        .with_child(button_toggle_menu);

    columns.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    let mut flex = Flex::<FractalData>::row()
        .with_flex_spacer(0.05)
        .with_flex_child(columns, 0.9)
        .with_flex_spacer(0.05);
    
    flex.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    let mut advanced_options_label = Label::<FractalData>::new("ADVANCED OPTIONS");
    advanced_options_label.set_text_size(20.0);

    let button_save_advanced_options = Button::new("SAVE & UPDATE").on_click(|ctx, data: &mut bool, _env| {
        // println!("{}", data.display_glitches);
        *data = false;
        ctx.submit_command(SET_ADVANCED_OPTIONS);
        // ctx.submit_command(Command::new(Selector::new("start_zoom_out"), (), Target::Auto));
    }).lens(FractalData::show_settings).expand_width().fix_height(40.0);

    let display_glitches = create_checkbox_row("Show glitched pixels").lens(FractalData::display_glitches);
    let experimental = create_checkbox_row("Use experimental algorithm").lens(FractalData::experimental);
    let jitter = create_checkbox_row("Jitter pixels").lens(FractalData::jitter);
    let auto_adjust_iterations = create_checkbox_row("Automatically adjust iterations").lens(FractalData::auto_adjust_iterations);
    let remove_centre = create_checkbox_row("Remove image centre").lens(FractalData::remove_centre);

    let advanced_options_title = Flex::<FractalData>::row()
        .with_flex_child(advanced_options_label.expand_width(), 0.8)
        .with_spacer(8.0)
        .with_flex_child(button_save_advanced_options, 0.2);

    let booleans_section = Flex::column()
        .with_child(display_glitches)
        .with_spacer(4.0)
        .with_child(experimental)
        .with_spacer(4.0)
        .with_child(jitter)
        .with_spacer(4.0)
        .with_child(auto_adjust_iterations)
        .with_spacer(4.0)
        .with_child(remove_centre);

    let order_section = create_label_slider_row("SERIES APPROXIMATION ORDER:", 280.0, 4.0, 128.0)
        .lens(FractalData::order.map(|val| *val as f64, |val, new| *val = new as i64));

    let glitch_tolerance_section = create_label_textbox_row("GLITCH TOLERANCE:", 280.0)
        .lens(FractalData::glitch_tolerance);

    let glitch_percentage_section = create_label_textbox_row("GLITCH PERCENTAGE:", 280.0)
        .lens(FractalData::glitch_percentage);

    let storage_interval_section = create_label_textbox_row("DATA STORAGE INTERVAL:", 280.0)
        .lens(FractalData::iteration_interval);

    let probe_sampling_section = create_label_textbox_row("PROBE SAMPLING:", 280.0)
        .lens(FractalData::probe_sampling);

    let jitter_factor_section = create_label_textbox_row("JITTER FACTOR:", 280.0)
        .lens(FractalData::jitter_factor);

    let values_section = Flex::column()
        .with_child(order_section)
        .with_spacer(4.0)
        .with_child(glitch_tolerance_section)
        .with_spacer(4.0)
        .with_child(glitch_percentage_section)
        .with_spacer(4.0)
        .with_child(storage_interval_section)
        .with_spacer(4.0)
        .with_child(probe_sampling_section)
        .with_spacer(4.0)
        .with_child(jitter_factor_section);

    let advanced_section = Flex::row()
        .with_flex_child(booleans_section, 0.5)
        .with_spacer(32.0)
        .with_flex_child(values_section, 0.5);

    let mut real_label = Label::<FractalData>::new("REAL:");
    let mut imag_label = Label::<FractalData>::new("IMAG:");
    real_label.set_text_size(14.0);
    imag_label.set_text_size(14.0);

    let real = LensWrap::new(TextBox::multiline().with_text_size(10.0).expand_width(), lens::RealLens);
    // let real = TextBox::multiline().with_text_size(10.0).expand_width().lens(FractalData::real);

    let imag = LensWrap::new(TextBox::multiline().with_text_size(10.0).expand_width(), lens::ImagLens);



    let real_row = Flex::row()
        .with_child(real_label.fix_width(60.0))
        .with_flex_child(real, 1.0);

    let imag_row = Flex::row()
        .with_child(imag_label.fix_width(60.0))
        .with_flex_child(imag, 1.0);

    let mut advanced_options = Flex::<FractalData>::column()
        .with_spacer(8.0)
        .with_child(advanced_options_title)
        .with_spacer(4.0)
        .with_child(advanced_section)
        .with_spacer(32.0)
        .with_child(real_row)
        .with_spacer(4.0)
        .with_child(imag_row);
        
    advanced_options.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);
    
    let mut advanced_options_flex = Flex::<FractalData>::row()
        .with_flex_spacer(0.05)
        .with_flex_child(advanced_options, 0.9)
        .with_flex_spacer(0.05);

    advanced_options_flex.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    // Advanced options
    // Display palette?
    // display glitches
    // glitch tolerance
    // glitch percentage
    // iteration interval
    // approximation order
    // experimental
    // probe sampling
    // jitter
    // auto adjust iterations?
    // remove center
    // maybe add jitter factor?

    let test_switcher = Either::new(|data: &FractalData, _env| {
            data.show_settings
        }, advanced_options_flex, render_screen);

    Split::columns(test_switcher, flex).split_point(0.75).draggable(true).solid_bar(true).bar_size(4.0)
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


    // let button_set_order = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
    //     ctx.submit_command(Command::new(Selector::new("set_approximation_order"), (), Target::Auto));
    // }).expand_width();