use druid::Selector;

pub const RESET_RENDERER_FAST: Selector<()> = Selector::new("reset_renderer_fast");
pub const RESET_RENDERER_FULL: Selector<()> = Selector::new("reset_renderer_full");

pub const MULTIPLY_ZOOM: Selector<f64> = Selector::new("multiply_zoom_level");
pub const TOGGLE_DERIVATIVE: Selector<()> = Selector::new("toggle_derivative");

pub const OPEN_LOCATION: Selector<()> = Selector::new("open_location");
pub const SAVE_LOCATION: Selector<()> = Selector::new("save_location");
pub const SAVE_ALL: Selector<()> = Selector::new("save_all");
pub const SAVE_IMAGE: Selector<()> = Selector::new("save_image");

pub const NATIVE_SIZE: Selector<()> = Selector::new("native_image_size");
pub const MULTIPLY_SIZE: Selector<f64> = Selector::new("multiply_image_size");
pub const SET_SIZE: Selector<(i64, i64)> = Selector::new("set_image_size");

pub const SET_ROTATION: Selector<f64> = Selector::new("set_rotation");
pub const SET_ITERATIONS: Selector<i64> = Selector::new("set_iterations");
pub const SET_LOCATION: Selector<()> = Selector::new("set_location");

pub const SET_OFFSET_DIVISION: Selector<()> = Selector::new("set_offset_division");

pub const SET_ADVANCED_OPTIONS: Selector<()> = Selector::new("set_advanced_options");

pub const UPDATE_PALETTE: Selector<()> = Selector::new("update_palette");
pub const STOP_RENDERING: Selector<()> = Selector::new("stop_rendering");

pub const REPAINT: Selector<()> = Selector::new("repaint");

pub const UPDATE_PROGRESS: Selector<(usize, f64, usize, usize, usize)> = Selector::new("update_progress");
pub const UPDATE_BUFFER: Selector<()> = Selector::new("update_buffer");

pub const ZOOM_OUT: Selector<()> = Selector::new("start_zoom_out");
pub const ZOOM_OUT_OPTIMISED: Selector<()> = Selector::new("start_zoom_out_optimised");

pub const THREAD_RESET_RENDERER_FULL: usize = 1;
pub const THREAD_RESET_RENDERER_FAST: usize = 2;