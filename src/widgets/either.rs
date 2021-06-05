use druid::widget::prelude::*;
use druid::{Point, WidgetPod};

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