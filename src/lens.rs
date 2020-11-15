use druid::Lens;

use crate::FractalData;

pub struct WidthLens;

impl Lens<FractalData, String> for WidthLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let mut string = data.temporary_width.to_string();
        if data.temporary_width == 0 {
            string = "".into();
        }
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_width.to_string();
        if data.temporary_width == 0 {
            string = "".into();
        }
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_width = string.parse().unwrap_or(0);
        v
    }
}

pub struct HeightLens;

impl Lens<FractalData, String> for HeightLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let mut string = data.temporary_height.to_string();
        if data.temporary_height == 0 {
            string = "".into();
        }
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_height.to_string();
        if data.temporary_height == 0 {
            string = "".into();
        }
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_height = string.parse().unwrap_or(0);
        v
    }
}

pub struct RealLens;

impl Lens<FractalData, String> for RealLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        f(&data.temporary_real)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let v = f(&mut data.temporary_real);
        data.temporary_real.retain(|c| {
            if c.is_digit(10) {
                true
            } else {
                match c {
                    'E' | 'e' | '.' | '-' => true,
                    _ => false
                }
            }
        });
        v
    }
}

pub struct ImagLens;

impl Lens<FractalData, String> for ImagLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        f(&data.temporary_imag)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let v = f(&mut data.temporary_imag);
        data.temporary_imag.retain(|c| {
            if c.is_digit(10) {
                true
            } else {
                match c {
                    'E' | 'e' | '.' | '-' => true,
                    _ => false
                }
            }
        });
        v
    }
}

pub struct ZoomLens;

impl Lens<FractalData, String> for ZoomLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        f(&data.temporary_zoom)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let v = f(&mut data.temporary_zoom);
        data.temporary_zoom.retain(|c| {
            if c.is_digit(10) {
                true
            } else {
                match c {
                    'E' | 'e' | '.' => true,
                    _ => false
                }
            }
        });
        v
    }
}

pub struct IterationLens;

impl Lens<FractalData, String> for IterationLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let string = data.temporary_iterations.to_string();
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_iterations.to_string();
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_iterations = string.parse().unwrap_or(0);
        v
    }
}

pub struct RotationLens;

impl Lens<FractalData, String> for RotationLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let string = data.temporary_rotation.to_string();
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_rotation.to_string();
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10) || c == '.');
        data.temporary_rotation = string.parse().unwrap_or(0.0);
        v
    }
}

pub struct OrderLens;

impl Lens<FractalData, String> for OrderLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        let string = data.temporary_order.to_string();
        f(&string)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let mut string = data.temporary_order.to_string();
        let v = f(&mut string);
        string.retain(|c| c.is_digit(10));
        data.temporary_order = string.parse().unwrap_or(0);
        v
    }
}