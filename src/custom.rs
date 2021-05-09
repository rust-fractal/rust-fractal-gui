use crate::{FractalData, commands::{UPDATE_PALETTE, UPDATE_PIXEL_INFORMATION}};

use druid::piet::{FontFamily, PietText, ImageFormat, ImageBuf};
use druid::widget::prelude::*;
use druid::{ArcStr, Color, FontDescriptor, Point, TextLayout, WidgetPod};
use druid::widget::{Controller, Image};

const LINE_HEIGHT_FACTOR: f64 = 1.2;
const X_PADDING: f64 = 5.0;
pub struct NoUpdateLabel {
    text: TextLayout<ArcStr>,
    // Does the layout need to be changed?
    needs_update: bool,
    font_size: f64
}

impl NoUpdateLabel {
    pub fn new(font_size: f64) -> NoUpdateLabel {
        NoUpdateLabel {
            text: TextLayout::new(),
            needs_update: true,
            font_size
        }
    }

    fn make_layout_if_needed(&mut self, value: &str, t: &mut PietText, env: &Env) {
        if self.needs_update {
            self.text
                .set_text(value.into());
            self.text
                .set_font(FontDescriptor::new(FontFamily::MONOSPACE).with_size(self.font_size));
            self.text.set_text_color(Color::WHITE);
            self.text.rebuild_if_needed(t, env);

            self.needs_update = false;
        }
    }
}

impl Widget<String> for NoUpdateLabel {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut String, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &String, _: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, _: &String, _: &String, _: &Env) {
        self.needs_update = true;
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &String, env: &Env) -> Size {
        self.make_layout_if_needed(&data, &mut ctx.text(), env);
        bc.constrain((
            self.text.size().width + 2.0 * X_PADDING,
            self.font_size * LINE_HEIGHT_FACTOR,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, env: &Env) {
        self.make_layout_if_needed(&data, &mut ctx.text(), env);
        let origin = Point::new(X_PADDING, 0.0);
        self.text.draw(ctx, origin);
    }
}

pub struct PaletteUpdateController;

impl Controller<FractalData, Image> for PaletteUpdateController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FractalData,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if command.is(UPDATE_PALETTE) => {
                let step = (data.buffer.lock().palette_interpolated_buffer.len() / 500).max(1);
                let raw_buffer = data.buffer.lock().palette_interpolated_buffer.iter().step_by(step).map(|value| {
                    let (r, g, b, _) = value.rgba_u8();

                    vec![r, g, b]
                }).flatten().collect::<Vec<u8>>();

                let width = raw_buffer.len() / 3;

                child.set_image_data(ImageBuf::from_raw(raw_buffer, ImageFormat::Rgb, width, 1))
            }
            other => child.event(ctx, other, data, env),
        }
    }
}

pub struct PixelInformationUpdateController;

impl Controller<FractalData, Image> for PixelInformationUpdateController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FractalData,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if command.is(UPDATE_PIXEL_INFORMATION) => {
                // child.set_image_data(ImageBuf::from_raw(data.pixel_rgb.lock().as_ref(), ImageFormat::Rgb, 15, 15));
                ctx.request_paint();
            }
            other => child.event(ctx, other, data, env),
        }
    }
}

/// A widget that switches between two possible child views.
pub struct Either<T> {
    closure: Box<dyn Fn(&T, &Env) -> usize>,
    branches: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    current: usize,
}

impl<T> Either<T> {
    /// Create a new widget that switches between two views.
    pub fn new(
        closure: impl Fn(&T, &Env) -> usize + 'static,
    ) -> Either<T> {
        Either {
            closure: Box::new(closure),
            branches: Vec::new(),
            current: 0,
        }
    }

    pub fn add_branch(mut self, branch: impl Widget<T> + 'static) -> Self {
        self.branches.push(WidgetPod::new(branch).boxed());
        self
    }
}

impl<T: Data> Widget<T> for Either<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if event.should_propagate_to_hidden() {
            for branch in &mut self.branches {
                branch.event(ctx, event, data, env);
            }
        } else {
            self.current_widget().event(ctx, event, data, env)
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.current = (self.closure)(data, env);
        }

        if event.should_propagate_to_hidden() {
            for branch in &mut self.branches {
                branch.lifecycle(ctx, event, data, env);
            }
        } else {
            self.current_widget().lifecycle(ctx, event, data, env)
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        let current = (self.closure)(data, env);
        if current != self.current {
            self.current = current;
            ctx.request_layout();
        }
        self.current_widget().update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let current_widget = self.current_widget();
        let size = current_widget.layout(ctx, bc, data, env);
        current_widget.set_origin(ctx, data, env, Point::ORIGIN);
        ctx.set_paint_insets(current_widget.paint_insets());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.current_widget().paint(ctx, data, env)
    }
}

impl<T> Either<T> {
    fn current_widget(&mut self) -> &mut WidgetPod<T, Box<dyn Widget<T>>> {
        &mut self.branches[self.current]
    }
}