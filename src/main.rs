mod domain;
mod conf;
mod infra;
mod services;
mod tcp;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use clap::Parser;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use services::app::app::App;
use crate::domain::cli::cli::Cli;

fn main() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    std::process::Command::new("clear").status().unwrap();
    let cli = Cli::parse();
    let path = cli.config.map_or_else(
        || vec![PathBuf::from("config.yaml")],
        |args| {
            args.into_iter().map(PathBuf::from).collect()
        }
    );
    let app = App::new(path);
    if let Err(err) = app.run() {
        log::error!("main | Error: {:#?}", err);
    };
}
