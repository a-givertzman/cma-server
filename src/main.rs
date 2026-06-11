mod domain;
mod conf;
mod infra;
mod services;
mod sync;
mod tcp;
#[cfg(test)]
mod tests;

pub use sal_core::error::Error;

use std::path::PathBuf;
use clap::Parser;
use debugging::session::debug_session::{DebugSession, LogLevel};
use sal_core::dbg::Dbg;
use services::app::app::App;
use crate::domain::cli::cli::Cli;

fn main() {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("sal_sync::thread_pool", LogLevel::Info)
        .init();
    let dbg = Dbg::own("main");
    if let Err(err) = std::process::Command::new("clear").status() {
        log::debug!("{dbg} | Can't clear terminal, error: {:?}", err);
    }
    let cli = Cli::parse();
    let path = cli.config.map_or_else(
        || vec![PathBuf::from("config.yaml")],
        |args| {
            args.into_iter().map(PathBuf::from).collect()
        }
    );
    let app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("{dbg} | Can't execute App, error: {:#?}", err);
    };
}
