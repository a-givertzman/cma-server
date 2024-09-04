//!
//! # Functions implemented for the Vibro-analysis
//! 
//! ## Input data
//! 
//! - Apmlitude sequences from vibro-sensor 
//! - Apmlitude sequences from accoustic-sensor 
//! 
//! ## Analysis
//! 
//! - FFT - converts input sequences from time domain into the frequency domain
//! - FFT filtering - removing periodical components
//! - Noise reduction filtering - Trend extruction
//! 
pub mod fn_va_fft;
pub mod unit_circle;
