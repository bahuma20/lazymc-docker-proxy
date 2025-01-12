#[macro_use]
extern crate log;

mod command;
mod docker;
mod kubernetes;
mod entrypoint;
mod health;
mod logging;

use clap::Parser;

/// Wrapper for lazymc to run against a docker minecraft server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Execute with this flag when running as a lazymc start command
    #[arg(short, long)]
    command: bool,

    /// The lazymc group name
    #[arg(short, long, requires_if("command", "true"))]
    group: Option<String>,

    /// Execute with this flag when running as a health check
    #[arg(short, long)]
    health: bool,

    // Can be one of "docker" or "kubernetes"
    #[arg(short, long, default_value = "docker")]
    backend: String,
}

/// Main entrypoint for the application
fn main() {
    logging::init();

    let args: Args = Args::parse();

    if args.command {
        command::run(args.group.unwrap(), &args.backend);
    } else if args.health {
        health::run();
    } else {
        entrypoint::run(&args.backend);
    }
}
