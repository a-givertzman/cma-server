use clap::Parser;
///
/// Application cli arguments
#[derive(Parser, Debug)]
#[command(version = "0.1.2", about = "CMA Server | Handling data on fly", long_about = None)]
pub struct Cli {
    ///
    /// Optional path to configuration files splitted with ' ', 
    /// if omitted 'config.yaml' from current dir will be used
    /// example: `cma-server --path my-conf.yaml`
    /// example: `cma-server --path my-conf.yaml cma-recorder.yaml`
    /// example: `cargo run --release -- --path my-conf.yaml cma-recorder.yaml`
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub config: Option<Vec<String>>,
}