![image](render.png)

<h1 align="center">rust-fractal-gui</h1>
<p align="center">
  <a href="https://github.com/rust-fractal/rust-fractal-gui/blob/master/LICENSE"><img src="https://img.shields.io/github/license/rust-fractal/rust-fractal-gui" alt="Repository License"></a>
  <a href="https://github.com/rust-fractal/rust-fractal-gui/"><img src="https://img.shields.io/tokei/lines/github/rust-fractal/rust-fractal-gui" alt="Repository Size"></a>
  <a href="https://github.com/rust-fractal/rust-fractal-gui/releases"><img src="https://img.shields.io/github/downloads/rust-fractal/rust-fractal-gui/total?style=flat" alt="Github Release"></a>
</p>

A GUI frontend for the rust-fractal library based on the Druid library. rust-fractal is a mandelbrot fractal renderer implementing both perturbation and series approximation. A reference point is iterated at high-precision, arbitrary precision and differences from this are calculated in machine precision. This allows for a large reduction in computation required to render an image, especially at high zoom levels. This generator features:

- Perturbation based iteration with glitch detection.
- Glitch correction through automatic reference movement and recalculation.
- Series approximation calculation to skip (and approximate) large amounts of perturbation iterations.
- Probe based method to determine series approximation skip.
- Multithreading of core loops through rayon.
- Configurable location and rendering options.
- Multiple save formats including PNG and EXR.
- Utilises scaling and mantissa-exponent based extended precision to allow for arbitrary zoom, whilst maintaining good performance. Verified to be working at depths exceeding E50000. This is theoretically only limited by MPFR's precision.

## Compiling
You need to be able to compile the 'rug' crate which requires a rust GNU toolchain. A nightly toolchain is also required. Look in the documentation for rug for more information on how to do this. Once all required dependencies have been installed, build the crate with:

```cargo build --release```

## Usage
Double click the executable. The file `start.toml` must be in the same directory so that the program is able to get the initial renderer settings. Some shortcuts are:

- `LCLICK` zoom in to mouse location
- `RCLICK` zoom out from center
- `Z` quick zoom into center
- `D` toggle rendering mode
- `O` open file
- `T` half rendering resolution
- `Y` double rendering resolution
- `N` native rendering resolution
- `R` rotate 15 degrees clockwise

## Acknowledgements
- claude (blog, Kalles Fraktaler 2+)
- pauldelbrot (glitch detection, nanoscope)
- knighty (superMB)
