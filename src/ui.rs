use druid::widget::{Label, Split, TextBox, Flex, Button, WidgetExt};
use druid::{Widget, Command, LensWrap, Selector};

use rust_fractal::renderer::FractalRenderer;

use config::{Config, File};

use crate::{FractalData, FractalWidget};

use crate::lens;

pub fn ui_builder() -> impl Widget<FractalData> {
    let mut settings = Config::default();
    settings.merge(File::with_name("start.toml")).unwrap();

    let render_screen = FractalWidget {
        renderer: FractalRenderer::new(settings.clone()),
    };

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
    let mut iterations_label = Label::<FractalData>::new("ITER: ");
    let mut rotation_label = Label::<FractalData>::new("ROTN: ");

    real_label.set_text_size(20.0);
    imag_label.set_text_size(20.0);
    zoom_label.set_text_size(20.0);
    iterations_label.set_text_size(20.0);
    rotation_label.set_text_size(20.0);

    let real = LensWrap::new(TextBox::new().expand_width(), lens::RealLens);
    let imag = LensWrap::new(TextBox::new().expand_width(), lens::ImagLens);
    let zoom = LensWrap::new(TextBox::new().expand_width(), lens::ZoomLens);
    let iterations = LensWrap::new(TextBox::new().expand_width(), lens::IterationLens);
    let rotation = LensWrap::new(TextBox::new().expand_width(), lens::RotationLens);

    let row_6 = Flex::row()
        .with_child(real_label)
        .with_flex_child(real, 1.0);

    let row_7 = Flex::row()
        .with_child(imag_label)
        .with_flex_child(imag, 1.0);

    let row_8 = Flex::row()
        .with_child(zoom_label)
        .with_flex_child(zoom, 1.0);

    let row_9 = Flex::row()
        .with_child(iterations_label)
        .with_flex_child(iterations, 1.0);

    let row_10 = Flex::row()
        .with_child(rotation_label)
        .with_flex_child(rotation, 1.0);

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
        ctx.submit_command(Command::new(Selector::new("save_location"), ()), None);
    }).expand_width();

    let row_11 = Flex::row()
        .with_flex_child(button_set_location, 1.0)
        .with_flex_child(button_zoom_in, 1.0)
        .with_flex_child(button_zoom_out, 1.0)
        .with_flex_child(button_load_location, 1.0)
        .with_flex_child(button_save_location, 1.0);

    let mut colouring_title = Label::<FractalData>::new("COLOURING");
    colouring_title.set_text_size(20.0);

    let row_12 = Flex::row()
        .with_flex_child(colouring_title.expand_width(), 1.0);

    let mut colouring_method_label = Label::<FractalData>::new("METHOD:   ");
    let mut palette_label = Label::<FractalData>::new("PALETTE:  ");
    let mut palette_offset_label = Label::<FractalData>::new("OFFSET:   ");
    let mut iteration_division_label = Label::<FractalData>::new("DIVISION: ");

    colouring_method_label.set_text_size(20.0);
    palette_label.set_text_size(20.0);
    iteration_division_label.set_text_size(20.0);
    palette_offset_label.set_text_size(20.0);

    let colouring = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();

        if settings.get_bool("analytic_derivative").unwrap() {
            "Distance".to_string()
        } else {
            "Iteration".to_string()
        }
    });

    let palette = Label::new(|data: &FractalData, _env: &_| {
        data.temporary_palette_source.clone()
    });

    let iteration_division = LensWrap::new(TextBox::new().expand_width(), lens::DivisionLens);
    let palette_offset = LensWrap::new(TextBox::new().expand_width(), lens::OffsetLens);

    let iteration_palette = Flex::column()
        .with_child(iteration_division_label)
        .with_child(palette_offset_label);
    
    let iteration_palette_2 = Flex::column()
        .with_child(iteration_division)
        .with_child(palette_offset);

    let set_iteration_offset = Button::new("SET").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_offset_division"), ()), None);
    });

    let row_13 = Flex::row()
        .with_child(colouring_method_label)
        .with_flex_child(colouring, 1.0);

    let row_14 = Flex::row()
        .with_child(palette_label)
        .with_flex_child(palette, 1.0);

    let row_15 = Flex::row()
        .with_child(iteration_palette)
        .with_flex_child(iteration_palette_2, 1.0)
        .with_child(set_iteration_offset);

    let row_16 = Flex::row();
        // .with_child(palette_offset_label)
        // .with_flex_child(palette_offset, 1.0);

    let button_set_method = Button::new("TOGGLE METHOD").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("toggle_derivative"), ()), None);
    }).expand_width();

    let button_set_palette = Button::new("LOAD PALETTE").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("open_location"), ()), None);
    }).expand_width();

    let row_17 = Flex::row()
        .with_flex_child(button_set_method, 1.0)
        .with_flex_child(button_set_palette, 1.0);

    let mut options_title = Label::<FractalData>::new("OPTIONS");
    options_title.set_text_size(20.0);

    let row_18 = Flex::row()
        .with_flex_child(options_title.expand_width(), 1.0);

    let mut order_label = Label::<FractalData>::new("ORDER:      ");

    order_label.set_text_size(20.0);

    let order = LensWrap::new(TextBox::new().expand_width(), lens::OrderLens);

    let row_19 = Flex::row()
        .with_child(order_label)
        .with_flex_child(order, 1.0);

    let button_set_order = Button::new("SET ORDER").on_click(|ctx, _data: &mut FractalData, _env| {
        ctx.submit_command(Command::new(Selector::new("set_approximation_order"), ()), None);
    }).expand_width();

    let row_20 = Flex::row()
        .with_flex_child(button_set_order, 1.0);

    

    let mut information_title = Label::<FractalData>::new("INFORMATION");
    information_title.set_text_size(20.0);

    let row_21 = Flex::row()
        .with_flex_child(information_title.expand_width(), 1.0);

    let mut min_skipped_label = Label::<FractalData>::new("SKIPPED: ");
    let mut render_time_label = Label::<FractalData>::new("RENDER:  ");

    min_skipped_label.set_text_size(20.0);
    render_time_label.set_text_size(20.0);

    let min_skipped = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();
        settings.get_int("min_valid_iteration").unwrap().to_string()
    });

    let render_time = Label::new(|data: &FractalData, _env: &_| {
        let settings = data.settings.lock().unwrap();
        format!("{} ms", settings.get_int("render_time").unwrap().to_string())
    });

    let row_22 = Flex::row()
        .with_child(min_skipped_label)
        .with_flex_child(min_skipped, 1.0);

    let row_23 = Flex::row()
        .with_child(render_time_label)
        .with_flex_child(render_time, 1.0);

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
        .with_child(row_10)
        .with_child(row_11)
        .with_spacer(8.0)
        .with_child(row_12)
        .with_child(row_13)
        .with_child(row_14)
        .with_child(row_15)
        .with_child(row_16)
        .with_child(row_17)
        .with_spacer(8.0)
        .with_child(row_18)
        .with_child(row_19)
        .with_child(row_20)
        .with_spacer(8.0)
        .with_child(row_21)
        .with_child(row_22)
        .with_child(row_23);

    columns.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);

    let mut flex = Flex::<FractalData>::row()
        .with_flex_spacer(0.1)
        .with_flex_child(columns, 0.8)
        .with_flex_spacer(0.1);
    
    flex.set_cross_axis_alignment(druid::widget::CrossAxisAlignment::Start);


    Split::columns(render_screen, flex).split_point(0.75).draggable(true)
}