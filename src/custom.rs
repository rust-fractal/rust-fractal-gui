
use crate::FractalData;

use druid::piet::{FontFamily, PietText, ImageFormat, ImageBuf};
use druid::widget::prelude::*;
use druid::{ArcStr, Color, FontDescriptor, Point, TextLayout, Selector};
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

            let formatted_time = format!("{}:{:0>2}:{:0>2}.{:0>3}", h, m % 60, s % 60, ms);

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
        // TODO: update on env changes also
        self.needs_update = true;
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &FractalData, env: &Env) -> Size {
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        self.make_layout_if_needed(data.temporary_time, data.temporary_stage, &mut ctx.text(), env);
        bc.constrain((
            self.text.size().width + 2.0 * X_PADDING,
            font_size * LINE_HEIGHT_FACTOR,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FractalData, env: &Env) {
        self.make_layout_if_needed(data.temporary_time, data.temporary_stage, &mut ctx.text(), env);
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
                if let Some(_) = command.get::<()>(Selector::new("update_palette")) {
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

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_data: &FractalData,
        data: &FractalData,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env);
    }
}