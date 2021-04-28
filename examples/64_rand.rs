#![no_std]
#![no_main]
#![feature(isa_attribute)]
/// This ROM prints number of ticks per set of random number generated
/// Used to compare the speed of the algo on the GBA, since GBA is a 
/// 32 bit system

use gba::{debug::{DebugInterface, DebugLevel, mgba::MGBADebugInterface}, prelude::*};

use randomize::Pcg32 as RNG;

const I_FLAGS: InterruptFlags = InterruptFlags::new()
  .with_vblank(true);

#[panic_handler]
#[allow(unused)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  loop {
    DISPCNT.read();
  }
}
fn setup_timer() {
  TIMER0_RELOAD.write(0u16);
  TIMER0_CONTROL.write(TimerControl::new().with_enabled(true).with_prescaler_selection(1));
}

fn setup_irq() {
  // Set the IRQ handler to use.
  unsafe { USER_IRQ_HANDLER.write(Some(irq_handler_a32)) };
  unsafe { IE.write(I_FLAGS) };
}
// Needed to stop the rng from being optimized away
#[used]
static mut X: u32 = 0;
#[no_mangle]
pub fn main() -> ! {
  const SETTING: DisplayControl = DisplayControl::new()
    .with_display_mode(3)
    .with_display_bg2(true);
  DISPCNT.write(SETTING);
 
  const DISPLAY_SETTINGS: DisplayStatus = DisplayStatus::new()
    .with_vblank_irq_enabled(true);
  
  DISPSTAT.write(DISPLAY_SETTINGS);

  let mut rng = RNG::seed(0, 0);
  setup_timer();
  setup_irq();

  let debug = MGBADebugInterface{};

  loop {
    let before = TIMER0_COUNTER.read();
    for _ in 0..1_000 {
      unsafe { X = rng.next_u32() };
      //mode3::bitmap_xy(mode3::WIDTH / 2, mode3::HEIGHT / 2).write(Color(x as u16));
    }
    let after = TIMER0_COUNTER.read();
    debug.debug_print(DebugLevel::Info, &format_args!("1,000 generations per {}*64 ticks", after - before ) ).unwrap();
    unsafe { VBlankIntrWait() };
  }
}


#[instruction_set(arm::a32)]
extern "C" fn irq_handler_a32() {
  // we just use this a32 function to jump over back to t32 code.
  irq_handler_t32()
}

fn irq_handler_t32() {
  // disable Interrupt Master Enable to prevent an interrupt during the handler
  unsafe { IME.write(false) };

  // read which interrupts are pending, and "filter" the selection by which are
  // supposed to be enabled.
  let which_interrupts_to_handle = IRQ_PENDING.read() & IE.read();

  // read the current IntrWait value. It sorta works like a running total, so
  // any interrupts we process we'll enable in this value, which we write back
  // at the end.
  let mut intr_wait_flags = INTR_WAIT_ACKNOWLEDGE.read();

  if which_interrupts_to_handle.vblank() {
    intr_wait_flags.set_vblank(true);
  }

  // acknowledge that we did stuff.
  IRQ_ACKNOWLEDGE.write(which_interrupts_to_handle);

  // write out any IntrWait changes.
  unsafe { INTR_WAIT_ACKNOWLEDGE.write(intr_wait_flags) };

  // re-enable as we go out.
  unsafe { IE.write(I_FLAGS) };
}