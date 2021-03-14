use crate::{FractalData, commands::{UPDATE_PALETTE, UPDATE_PIXEL_INFORMATION}};

use druid::piet::{FontFamily, PietText, ImageFormat, ImageBuf};
use druid::widget::prelude::*;
use druid::{ArcStr, Color, FontDescriptor, Point, TextLayout};
use druid::widget::{Controller, Image};
use druid::{Data, WidgetPod};

const LINE_HEIGHT_FACTOR: f64 = 1.2;
const X_PADDING: f64 = 5.0;
pub struct NoUpdateLabel {
    text: TextLayout<ArcStr>,
    // Does the layout need to be changed?
    needs_update: bool,
}

impl NoUpdateLabel {
    pub fn new() -> NoUpdateLabel {
        NoUpdateLabel {
            text: TextLayout::new(),
            needs_update: true,
        }
    }

    fn make_layout_if_needed(&mut self, value: &String, t: &mut PietText, env: &Env) {
        if self.needs_update {
            let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);

            self.text
                .set_text(value.clone().into());
            self.text
                .set_font(FontDescriptor::new(FontFamily::MONOSPACE).with_size(font_size));
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
        // println!("timer update");
        // TODO: update on env changes also
        self.needs_update = true;
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &String, env: &Env) -> Size {
        // println!("timer layout");
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        self.make_layout_if_needed(&data, &mut ctx.text(), env);
        bc.constrain((
            self.text.size().width + 2.0 * X_PADDING,
            font_size * LINE_HEIGHT_FACTOR,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, env: &Env) {
        // println!("timer paint");
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
                let raw_buffer = data.buffer.lock().palette_generator.colors(100).iter().map(|value| {
                    let (r, g, b, _) = value.rgba_u8();

                    vec![r, g, b]
                }).flatten().collect::<Vec<u8>>();

                child.set_image_data(ImageBuf::from_raw(raw_buffer, ImageFormat::Rgb, 100, 1))
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
                child.set_image_data(ImageBuf::from_raw(data.pixel_rgb.lock().as_ref(), ImageFormat::Rgb, 15, 15));
                ctx.request_paint();
            }
            other => child.event(ctx, other, data, env),
        }
    }
}

/// A widget that switches between two possible child views.
pub struct Either<T> {
    closure: Box<dyn Fn(&T, &Env) -> bool>,
    true_branch: WidgetPod<T, Box<dyn Widget<T>>>,
    false_branch: WidgetPod<T, Box<dyn Widget<T>>>,
    current: bool,
}

impl<T> Either<T> {
    /// Create a new widget that switches between two views.
    ///
    /// The given closure is evaluated on data change. If its value is `true`, then
    /// the `true_branch` widget is shown, otherwise `false_branch`.
    pub fn new(
        closure: impl Fn(&T, &Env) -> bool + 'static,
        true_branch: impl Widget<T> + 'static,
        false_branch: impl Widget<T> + 'static,
    ) -> Either<T> {
        Either {
            closure: Box::new(closure),
            true_branch: WidgetPod::new(true_branch).boxed(),
            false_branch: WidgetPod::new(false_branch).boxed(),
            current: false,
        }
    }
}

impl<T: Data> Widget<T> for Either<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // println!("{:?}", event);
        if event.should_propagate_to_hidden() {
            self.true_branch.event(ctx, event, data, env);
            self.false_branch.event(ctx, event, data, env);
        } else {
            self.current_widget().event(ctx, event, data, env)
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.current = (self.closure)(data, env);
        }

        if event.should_propagate_to_hidden() {
            // println!("hidden");
            self.true_branch.lifecycle(ctx, event, data, env);
            self.false_branch.lifecycle(ctx, event, data, env);
        } else {
            self.current_widget().lifecycle(ctx, event, data, env)
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        // println!("evaluating closure");
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
        if self.current {
            &mut self.true_branch
        } else {
            &mut self.false_branch
        }
    }
}