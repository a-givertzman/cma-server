use std::path::PathBuf;
use clap::Parser;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use services::app::app::App;
use crate::core_::cli::cli::Cli;

#[cfg(test)]
mod tests;
mod core_;
mod conf;
mod services;
mod tcp;

fn main() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
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
