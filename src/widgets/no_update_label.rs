use druid::piet::{FontFamily, PietText};
use druid::widget::prelude::*;
use druid::{ArcStr, Color, FontDescriptor, Point, TextLayout};

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