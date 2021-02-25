use crate::{FractalData, commands::UPDATE_PALETTE};

use druid::piet::{FontFamily, PietText, ImageFormat, ImageBuf};
use druid::widget::prelude::*;
use druid::{ArcStr, Color, FontDescriptor, Point, TextLayout};
use druid::widget::{Controller, Image};

const LINE_HEIGHT_FACTOR: f64 = 1.2;
const X_PADDING: f64 = 5.0;

pub struct RenderTimer {
    text: TextLayout<ArcStr>,
    // Does the layout need to be changed?
    needs_update: bool,
}

impl RenderTimer {
    pub fn new() -> RenderTimer {
        RenderTimer {
            text: TextLayout::new(),
            needs_update: true,
        }
    }

    fn make_layout_if_needed(&mut self, time: usize, stage: usize, t: &mut PietText, env: &Env) {
        if self.needs_update {
            let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);

            let text = match stage {
                1 => "REFERENCE",
                2 => "APPROXIMATION",
                3 => "ITERATION",
                4 => "CORRECTION",
                0 => "COMPLETE",
                _ => "DEFAULT"
            };

            let ms = time % 1000;
            let s = time / 1000;
            let m = s / 60;
            let h = m / 60;

            let formatted_time = format!("{}:{:0>2}:{:0>2}:{:0>3}", h, m % 60, s % 60, ms);

            self.text
                .set_text(format!("{:>14} {:>14}", text, formatted_time).into());
            self.text
                .set_font(FontDescriptor::new(FontFamily::MONOSPACE).with_size(font_size));
            self.text.set_text_color(Color::WHITE);
            self.text.rebuild_if_needed(t, env);

            self.needs_update = false;
        }
    }
}

impl Widget<FractalData> for RenderTimer {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut FractalData, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &FractalData, _: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, _: &FractalData, _: &FractalData, _: &Env) {
        // println!("timer update");
        // TODO: update on env changes also
        self.needs_update = true;
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FractalData, env: &Env) -> Size {
        // println!("timer layout");
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        self.make_layout_if_needed(data.temporary_time, data.temporary_stage, &mut ctx.text(), env);
        bc.constrain((
            self.text.size().width + 2.0 * X_PADDING,
            font_size * LINE_HEIGHT_FACTOR,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, env: &Env) {
        // println!("timer paint");
        self.make_layout_if_needed(data.temporary_time, data.temporary_stage, &mut ctx.text(), env);
        let origin = Point::new(X_PADDING, 0.0);
        self.text.draw(ctx, origin);
    }
}

pub struct SkippedLabel {
    text: TextLayout<ArcStr>,
    // Does the layout need to be changed?
    needs_update: bool,
}

impl SkippedLabel {
    pub fn new() -> SkippedLabel {
        SkippedLabel {
            text: TextLayout::new(),
            needs_update: true,
        }
    }

    fn make_layout_if_needed(&mut self, min_skipped_iterations: usize, max_skipped_iterations: usize, t: &mut PietText, env: &Env) {
        if self.needs_update {
            let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);

            let temp = format!("min. {} max. {}", min_skipped_iterations, max_skipped_iterations);

            self.text
                .set_text(format!("{:>30}", temp).into());
            self.text
                .set_font(FontDescriptor::new(FontFamily::MONOSPACE).with_size(font_size));
            self.text.set_text_color(Color::WHITE);
            self.text.rebuild_if_needed(t, env);

            self.needs_update = false;
        }
    }
}

impl Widget<FractalData> for SkippedLabel {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut FractalData, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &FractalData, _: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, _: &FractalData, _: &FractalData, _: &Env) {
        // println!("timer update");
        // TODO: update on env changes also
        self.needs_update = true;
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FractalData, env: &Env) -> Size {
        // println!("timer layout");
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        self.make_layout_if_needed(data.temporary_min_valid_iterations, data.temporary_max_valid_iterations, &mut ctx.text(), env);
        bc.constrain((
            self.text.size().width + 2.0 * X_PADDING,
            font_size * LINE_HEIGHT_FACTOR,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, env: &Env) {
        // println!("timer paint");
        self.make_layout_if_needed(data.temporary_min_valid_iterations, data.temporary_max_valid_iterations, &mut ctx.text(), env);
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
            Event::Command(command) => {
                if command.is(UPDATE_PALETTE) {
                    let settings = data.settings.lock().unwrap();

                    let raw_buffer = settings.get_array("palette").unwrap().chunks(3).map(|value| {
                        Vec::from([value[2].clone().into_int().unwrap() as u8, value[1].clone().into_int().unwrap() as u8, value[0].clone().into_int().unwrap() as u8])
                    }).flatten().collect::<Vec<u8>>();
                
                    let test = ImageBuf::from_raw(raw_buffer.clone(), ImageFormat::Rgb, raw_buffer.len() / 3, 1);

                    child.set_image_data(test)
                }
            }
            other => child.event(ctx, other, data, env),
        }
    }
}

use druid::{Data, WidgetPod};

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