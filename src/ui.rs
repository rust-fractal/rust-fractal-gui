use druid::widget::{Label, Split, TextBox, Flex, Button, WidgetExt, ProgressBar};
use druid::{Widget, Command, LensWrap, Selector};

use config::{Config, File};

use crate::{FractalData, FractalWidget};

use crate::lens;

pub fn ui_builder() -> impl Widget<FractalData> {
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    let render_screen = FractalWidget {
        buffer: Vec::new(),
        reset_buffer: false,
        image_width: 0,
        image_height: 0
    };

    let mut resolution_title = Label::<FractalData>::new("RESOLUTION");
    resolution_title.set_text_size(20.0);

    let row_1 = Flex::row()
        .with_flex_child(resolution_title.expand_width(), 1.0);

    let mut width_label = Label::<FractalData>::new("WIDTH:  ");
    let mut height_label = Label::<FractalData>::new("HEIGHT: ");

    width_label.set_text_size(14.0);
    height_label.set_text_size(14.0);

    let image_width = LensWrap::new(TextBox::new().expand_width(), lens::WidthLens);
    let image_height = LensWrap::new(TextBox::new().expand_width(), lens::HeightLens);

    let button_set_image_size = Button::new("SET").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_image_size"), (data.temporary_width, data.temporary_height)), None);
    }).expand_width().fix_height(36.0);

    let image_width_row = Flex::row()
        .with_child(width_label)
        .with_flex_child(image_width, 1.0);

    let image_height_row = Flex::row()
        .with_child(height_label)
        .with_flex_child(image_height, 1.0);

    let image_size_column = Flex::column()
        .with_child(image_width_row)
        .with_spacer(4.0)
        .with_child(image_height_row);

    let row_2 = Flex::row()
        .with_flex_child(image_size_column, 0.75)
        .with_spacer(4.0)
        .with_flex_child(button_set_image_size, 0.25);

    let button_half = Button::new("HALF").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 0.5), None);
    }).expand_width();

    let button_double = Button::new("DOUBLE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_image_size"), 2.0), None);
    }).expand_width();

    let button_native = Button::new("NATIVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("native_image_size"), ()), None);
    }).expand_width();

    let row_3 = Flex::row()
        .with_flex_child(button_half, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_double, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_native, 1.0);

    let mut location_title = Label::<FractalData>::new("LOCATION");
    location_title.set_text_size(20.0);

    let location = Label::new(|data: &FractalData, _env: &_| {
        data.temporary_location_source.clone()
    });

    let row_4 = Flex::row()
        .with_flex_child(location_title.expand_width(), 0.5)
        .with_flex_child(location.expand_width(), 0.5);

    let mut real_label = Label::<FractalData>::new("REAL: ");
    let mut imag_label = Label::<FractalData>::new("IMAG: ");
    let mut zoom_label = Label::<FractalData>::new("ZOOM: ");
    let mut iterations_label = Label::<FractalData>::new("ITER: ");
    let mut rotation_label = Label::<FractalData>::new("ROTN: ");

    real_label.set_text_size(14.0);
    imag_label.set_text_size(14.0);
    zoom_label.set_text_size(14.0);
    iterations_label.set_text_size(14.0);
    rotation_label.set_text_size(14.0);

    let real = LensWrap::new(TextBox::new().expand_width(), lens::RealLens);
    let imag = LensWrap::new(TextBox::new().expand_width(), lens::ImagLens);
    let zoom = LensWrap::new(TextBox::new().expand_width(), lens::ZoomLens);
    let iterations = LensWrap::new(TextBox::new().expand_width(), lens::IterationLens);
    let rotation = LensWrap::new(TextBox::new().expand_width(), lens::RotationLens);

    let real_row = Flex::row()
        .with_child(real_label)
        .with_flex_child(real, 1.0);

    let imag_row = Flex::row()
        .with_child(imag_label)
        .with_flex_child(imag, 1.0);

    let button_zoom_in = Button::new("+").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 2.0), None);
    }).expand_width();

    let button_zoom_out = Button::new("-").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("multiply_zoom_level"), 0.5), None);
    }).expand_width();

    let zoom_row = Flex::row()
        .with_child(zoom_label)
        .with_flex_child(zoom, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_zoom_in, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_zoom_out, 0.15);

    let button_increase_iterations = Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_iterations"), 2 * data.temporary_iterations), None);
    }).expand_width();

    let button_decrease_iterations = Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_iterations"), data.temporary_iterations / 2), None);
    }).expand_width();

    let iterations_row = Flex::row()
        .with_child(iterations_label)
        .with_flex_child(iterations, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_increase_iterations, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_decrease_iterations, 0.15);

    let button_increase_rotation = Button::new("+").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_rotation"), data.temporary_rotation - 15.0), None);
    }).expand_width();

    let button_decrease_rotation = Button::new("-").on_click(|ctx, data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_rotation"), data.temporary_rotation + 15.0), None);
    }).expand_width();

    let rotation_row = Flex::row()
        .with_child(rotation_label)
        .with_flex_child(rotation, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_increase_rotation, 0.15)
        .with_spacer(2.0)
        .with_flex_child(button_decrease_rotation, 0.15);

    let row_5 = Flex::column()
        .with_child(real_row)
        .with_spacer(4.0)
        .with_child(imag_row)
        .with_spacer(4.0)
        .with_child(zoom_row)
        .with_spacer(4.0)
        .with_child(iterations_row)
        .with_spacer(4.0)
        .with_child(rotation_row);

    let button_set_location = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_location"), ()), None);
    }).expand_width();

    let button_load_location = Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("open_location"), ()), None);
    }).expand_width();

    let button_save_location = Button::new("SAVE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("save_location"), ()), None);
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

    let mut colouring_method_label = Label::<FractalData>::new("METHOD:   ");
    let mut palette_label = Label::<FractalData>::new("PALETTE:  ");
    let mut iteration_division_label = Label::<FractalData>::new("DIVISION: ");
    let mut palette_offset_label = Label::<FractalData>::new("OFFSET:   ");

    colouring_method_label.set_text_size(14.0);
    palette_label.set_text_size(14.0);
    iteration_division_label.set_text_size(14.0);
    palette_offset_label.set_text_size(14.0);

    let mut colouring = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();

        if settings.get_bool("analytic_derivative").unwrap() {
            "distance".to_string()
        } else {
            "iteration".to_string()
        }
    });

    let mut palette = Label::new(|data: &FractalData, _env: &_| {
        data.temporary_palette_source.clone()
    });

    colouring.set_text_size(12.0);
    palette.set_text_size(8.0);

    let iteration_division = LensWrap::new(TextBox::new().expand_width(), lens::DivisionLens);
    let palette_offset = LensWrap::new(TextBox::new().expand_width(), lens::OffsetLens);

    let button_set_method = Button::new("TOGGLE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("toggle_derivative"), ()), None);
    }).expand_width();

    let colouring_method_row = Flex::row()
        .with_child(colouring_method_label)
        .with_flex_child(colouring.expand_width(), 0.6)
        .with_spacer(2.0)
        .with_flex_child(button_set_method, 0.4);

    let button_set_palette = Button::new("LOAD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("open_location"), ()), None);
    }).expand_width();
    
    let palette_row = Flex::row()
        .with_child(palette_label)
        .with_flex_child(palette.expand_width(), 0.6)
        .with_spacer(2.0)
        .with_flex_child(button_set_palette, 0.4);

    let iteration_division_row = Flex::row()
        .with_child(iteration_division_label)
        .with_flex_child(iteration_division, 1.0);

    let palette_offset_row = Flex::row()
        .with_child(palette_offset_label)
        .with_flex_child(palette_offset, 1.0);

    let set_iteration_offset = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_offset_division"), ()), None);
    }).expand_width().fix_height(36.0);

    let offset_division_column = Flex::column()
        .with_child(iteration_division_row)
        .with_spacer(4.0)
        .with_child(palette_offset_row);

    let offset_division_row = Flex::row()
        .with_flex_child(offset_division_column, 0.7)
        .with_spacer(4.0)
        .with_flex_child(set_iteration_offset, 0.3);

    let row_8 = Flex::column()
        .with_child(colouring_method_row)
        .with_spacer(4.0)
        .with_child(palette_row)
        .with_spacer(4.0)
        .with_child(offset_division_row);

    let mut options_title = Label::<FractalData>::new("OPTIONS");
    options_title.set_text_size(20.0);

    let row_9 = Flex::row()
        .with_flex_child(options_title.expand_width(), 1.0);

    let mut order_label = Label::<FractalData>::new("ORDER: ");

    order_label.set_text_size(14.0);

    let order = LensWrap::new(TextBox::new().expand_width(), lens::OrderLens);

    let button_set_order = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_approximation_order"), ()), None);
    }).expand_width();

    let row_10 = Flex::row()
        .with_child(order_label)
        .with_flex_child(order, 0.7)
        .with_spacer(4.0)
        .with_flex_child(button_set_order, 0.3);

    let button_save_image = Button::new("SAVE IMAGE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("save_image"), ()), None);
    }).expand_width();

    let button_refresh_full = Button::new("RESET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("reset_renderer_full"), ()), None);
    }).expand_width();

    let row_11 = Flex::row()
        .with_flex_child(button_save_image, 1.0)
        .with_spacer(2.0)
        .with_flex_child(button_refresh_full, 1.0);

    let mut information_title = Label::<FractalData>::new("INFORMATION");
    information_title.set_text_size(20.0);

    let row_12 = Flex::row()
        .with_flex_child(information_title.expand_width(), 1.0);

    // TODO maybe make these update live with the rendering progress
    let mut min_skipped_label = Label::<FractalData>::new("SKIPPED: ");
    let mut render_time_label = Label::<FractalData>::new("RENDER:  ");

    min_skipped_label.set_text_size(14.0);
    render_time_label.set_text_size(14.0);

    let min_skipped = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();
        settings.get_int("min_valid_iteration").unwrap().to_string()
    });

    let render_time = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();
        format!("{} ms", settings.get_int("render_time").unwrap().to_string())
    });

    let row_13 = Flex::row()
        .with_child(min_skipped_label)
        .with_flex_child(min_skipped, 1.0);

    let row_14 = Flex::row()
        .with_child(render_time_label)
        .with_flex_child(render_time, 1.0);

    let render_stage = Label::dynamic(|data: &FractalData, _| {
        let text = match data.temporary_stage {
            0 => "REFERENCE".to_string(),
            1 => "ITERATION".to_string(),
            2 => "CORRECTION".to_string(),
            3 => "COMPLETE".to_string(),
            _ => "DEFAULT".to_string()
        };

        format!("{:<15}", text)
    });

    let render_progress = LensWrap::new(ProgressBar::new().expand_width(), FractalData::temporary_progress1);

    let row_15 = Flex::row()
        .with_child(render_stage)
        .with_flex_child(render_progress, 1.0);

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
        .with_child(row_10)
        .with_spacer(4.0)
        .with_child(row_11)
        .with_spacer(8.0)
        .with_child(row_12)
        .with_child(row_13)
        .with_child(row_14)
        .with_child(row_15);

    columns.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    let mut flex = Flex::<FractalData>::row()
        .with_flex_spacer(0.05)
        .with_flex_child(columns, 0.9)
        .with_flex_spacer(0.05);
    
    flex.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);


    Split::columns(render_screen, flex).split_point(0.75).draggable(true)
}