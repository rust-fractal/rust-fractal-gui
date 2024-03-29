use druid::Selector;

use rust_fractal::util::{ComplexExtended, FloatExtended, data_export::ColoringType};

pub const RESET_RENDERER_FAST: Selector<()> = Selector::new("reset_renderer_fast");
pub const RESET_RENDERER_FULL: Selector<()> = Selector::new("reset_renderer_full");

pub const MULTIPLY_ZOOM: Selector<f64> = Selector::new("multiply_zoom_level");
pub const SET_COLORING_METHOD: Selector<ColoringType> = Selector::new("set_coloring_method");

pub const OPEN_LOCATION: Selector<()> = Selector::new("open_location");
pub const SAVE_LOCATION: Selector<()> = Selector::new("save_location");
pub const SAVE_ALL: Selector<()> = Selector::new("save_all");
pub const SAVE_IMAGE: Selector<()> = Selector::new("save_image");

pub const NATIVE_SIZE: Selector<()> = Selector::new("native_image_size");
pub const MULTIPLY_SIZE: Selector<f64> = Selector::new("multiply_image_size");
pub const SET_SIZE: Selector<(usize, usize)> = Selector::new("set_image_size");

pub const MULTIPLY_PATTERN: Selector<f64> = Selector::new("multiply_pattern");

pub const SET_ROTATION: Selector<f64> = Selector::new("set_rotation");
pub const SET_ITERATIONS: Selector<usize> = Selector::new("set_iterations");
pub const SET_LOCATION: Selector<()> = Selector::new("set_location");

pub const REVERT_LOCATION: Selector<()> = Selector::new("revert_location");

pub const SET_OFFSET_SPAN: Selector<()> = Selector::new("set_offset_division");

pub const SET_PERIOD: Selector<usize> = Selector::new("set_period");
pub const ROOT_FINDING_COMPLETE: Selector<Option<FloatExtended>> = Selector::new("root_finding_complete");

pub const SET_ADVANCED_OPTIONS: Selector<()> = Selector::new("set_advanced_options");

pub const UPDATE_PALETTE: Selector<()> = Selector::new("update_palette");
pub const UPDATE_PIXEL_INFORMATION: Selector<()> = Selector::new("update_pixel_information");

pub const STOP_RENDERING: Selector<()> = Selector::new("stop_rendering");
pub const STOP_ROOT_FINDING: Selector<()> = Selector::new("step_root_finding");

pub const REPAINT: Selector<()> = Selector::new("repaint");
pub const RESET_DEFAULT_LOCATION: Selector<()> = Selector::new("reset_default_location");

pub const UPDATE_RENDERING_PROGRESS: Selector<(usize, f64, usize, usize, usize, usize)> = Selector::new("update_rendering_progress");
pub const UPDATE_ROOT_PROGRESS: Selector<(usize, usize, ComplexExtended)> = Selector::new("update_root_progress");

pub const ZOOM_OUT: Selector<()> = Selector::new("start_zoom_out");
pub const ZOOM_OUT_OPTIMISED: Selector<()> = Selector::new("start_zoom_out_optimised");

pub const CALCULATE_ROOT: Selector<()> = Selector::new("calculate_root");

pub const THREAD_RESET_RENDERER_FULL: usize = 1;
pub const THREAD_RESET_RENDERER_FAST: usize = 2;
pub const THREAD_CALCULATE_ROOT: usize = 3;

