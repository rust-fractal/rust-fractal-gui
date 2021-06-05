use crate::{widgets::FractalData, commands::{UPDATE_PALETTE, UPDATE_PIXEL_INFORMATION}};

use druid::piet::{ImageFormat, ImageBuf};
use druid::widget::prelude::*;
use druid::widget::{Controller, Image};

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