#![no_std]
#![no_main]

use gba::prelude::*;

use randomize::Gen32;

use randomize::RNG;

#[panic_handler]
#[allow(unused)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  // This kills the emulation with a message if we're running inside an
  // emulator we support (mGBA or NO$GBA), or just crashes the game if we
  // aren't.
  //fatal!("{}", info);

  loop {
    DISPCNT.read();
  }
}

/// Performs a busy loop until VBlank starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
pub fn spin_until_vblank() {
  while VCOUNT.read() < 160 {}
}

/// Performs a busy loop until VDraw starts.
///
/// This is very inefficient, and please keep following the lessons until we
/// cover how interrupts work!
pub fn spin_until_vdraw() {
  while VCOUNT.read() >= 160 {}
}

#[no_mangle]
pub fn main() -> ! {
  const SETTING: DisplayControl = DisplayControl::new().with_display_mode(3).with_display_bg2(true);
  DISPCNT.write(SETTING);
  // let mut rng = Pcg32::seed(SEED, 0);
  let mut rng = RNG::seed(0, 0);

  const COLORS: [Color; 6] = [
    Color::from_rgb(31,0,0), // Red
    Color::from_rgb(0,31,0), // Green
    Color::from_rgb(0,0,31), // Blue
    Color::from_rgb(31,0,31),
    Color::from_rgb(31, 31, 0),
    Color::from_rgb(31,31,31), // White
  ];

  let mut px: usize = 0;
  let mut py: usize = 0;
  let mut color = rng.pick(&COLORS);

  loop {
    
    // now we wait
    spin_until_vblank();
    mode3::bitmap_xy(px, py).write(color);
    mode3::bitmap_xy(px, py + 1).write(color);
    mode3::bitmap_xy(px + 1, py).write(color);
    mode3::bitmap_xy(px + 1, py + 1).write(color);
    px += 2;
    if px >= mode3::WIDTH {
      px = 0;
      py += 2;
      if py >= mode3::HEIGHT {
        py = 0;
      }
    }
    color = rng.pick(&COLORS);
    
    // now we wait again
    spin_until_vdraw();
  }
}