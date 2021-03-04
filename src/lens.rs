use druid::Lens;

use crate::FractalData;
pub struct RealLens;

impl Lens<FractalData, String> for RealLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        f(&data.real)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let v = f(&mut data.real);
        data.real.retain(|c| {
            if c.is_digit(10) {
                true
            } else {
                matches!(c, 'E' | 'e' | '.' | '-')
            }
        });

        data.real = data.real.to_uppercase();

        v
    }
}

pub struct ImagLens;

impl Lens<FractalData, String> for ImagLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &FractalData, f: F) -> V {
        f(&data.imag)
    }
    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut FractalData, f: F) -> V {
        let v = f(&mut data.imag);
        data.imag.retain(|c| {
            if c.is_digit(10) {
                true
            } else {
                matches!(c, 'E' | 'e' | '.' | '-')
            }
        });

        data.imag = data.imag.to_uppercase();

        v
    }
}