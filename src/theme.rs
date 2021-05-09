use druid::{Color, Env, FontDescriptor, FontFamily};
use druid::theme::{
    TEXT_SIZE_NORMAL, 
    UI_FONT, 
    TEXTBOX_BORDER_RADIUS,
    TEXTBOX_BORDER_WIDTH,
    PROGRESS_BAR_RADIUS, 
    BORDERED_WIDGET_HEIGHT, 
    // BORDER_DARK,
    PRIMARY_LIGHT, 
    PRIMARY_DARK, 
    BACKGROUND_LIGHT, 
    BACKGROUND_DARK, 
    WINDOW_BACKGROUND_COLOR,
    BUTTON_BORDER_RADIUS,
    BUTTON_BORDER_WIDTH,
    BUTTON_LIGHT,
    BUTTON_DARK,
    // BASIC_WIDGET_HEIGHT
};

pub fn configure_env(env: &mut Env) {
    env.set(UI_FONT, FontDescriptor::new(FontFamily::new_unchecked("lucida console")));
    env.set(TEXT_SIZE_NORMAL, 12.0);

    env.set(BUTTON_BORDER_RADIUS, 0.0);
    env.set(TEXTBOX_BORDER_RADIUS, 0.0);
    env.set(PROGRESS_BAR_RADIUS, 0.0);

    env.set(BUTTON_BORDER_WIDTH, 1.5);
    env.set(TEXTBOX_BORDER_WIDTH, 1.5);

    env.set(BORDERED_WIDGET_HEIGHT, 12.0);

    // env.set(BASIC_WIDGET_HEIGHT, 10.0);

    // env.set(BORDER_DARK, Color::from_hex_str("#191414").unwrap());

    env.set(PRIMARY_LIGHT, Color::from_hex_str("#1DB954").unwrap());
    env.set(PRIMARY_DARK, Color::from_hex_str("#1DB954").unwrap());

    env.set(BACKGROUND_LIGHT, Color::from_hex_str("#191414").unwrap());
    env.set(BACKGROUND_DARK, Color::from_hex_str("#191414").unwrap());

    env.set(BUTTON_LIGHT, Color::from_hex_str("#3F3F3F").unwrap());
    env.set(BUTTON_DARK, Color::from_hex_str("#3F3F3F").unwrap());

    env.set(WINDOW_BACKGROUND_COLOR, Color::from_hex_str("#191414").unwrap());
}