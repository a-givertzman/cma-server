//!
//! # `Task` Service | Time based functions
//! 
//! ## `FnTimerOnDelay` (TON)
//! 
//! **Delayed activation of the output.**
//! 
//! - Retirns TRUE only after the Input has remained TRUE for the specified duration.
//! - If Input drops to FALSE at any time, Output immediately resets to FALSE and the timer clears.
//! - Useful for signals debouncing, defining persistent conditions or staggering
//! 
//! 
//! ## `FnTimerOffDelay` (TOF)
//! 
//! **Extends the duration of a signal after it ends.**
//! 
//! - Returns TRUE immediately when the Input has TRUE.
//! - When Input becomes to FALSE, Output remains TRUE for the specified duration.
//! - If Input becomes TRUE again during the cooldown, the timer resets and Output stays TRUE.
//! - Useful for cool-down device, interior lighting delays, or maintaining a state during brief signal dropouts.
//! 
//! ## FnTimerPulse (TP)
//! 
//! **Generates a single pulse of a fixed length.**
//! 
//! - Returns TRUE for the specified duration only after Input becomes to TRUE,
//! regardless of how long Input stays TRUE or if it drops to FALSE early.
//! - The timer is non-retriggerable; it must finish the pulse before it can be started again.
//! - Useful for consistent trigger pulses, valve pulsing, or triggering a "one-shot" physical action.
//! 
//! ## FnTimer
//! 
//! **Measures the time while Input is TRUE**
//! 
//! - Returns time elapsed in seconds (double) since Input raised (>0) to dropped (<=0)
//! - If option `repeat` = true, then returns total elapsed secods of multiple periods
//! 

mod fn_timer_on_delay;
mod fn_timer;

pub use fn_timer_on_delay::*;
pub use fn_timer::*;
