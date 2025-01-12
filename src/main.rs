#[macro_use]
extern crate log;

mod command;
mod docker;
mod kubernetes;
mod entrypoint;
mod health;
mod logging;

use std::env::var;
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
}

#[derive(Clone)]
enum BackendType {
    Docker,
    Kubernetes,
}

/// Main entrypoint for the application
fn main() {
    logging::init();

    let args: Args = Args::parse();

    let backend_type_key;

    if let Ok(value) = var("BACKEND_TYPE") {
        backend_type_key = value.clone();
    } else {
        backend_type_key = "docker".to_string();
    }

    let backend_type = match backend_type_key.as_str() {
        "docker" => BackendType::Docker,
        "kubernetes" => BackendType::Kubernetes,
        &_ => { panic!("Invalid BACKEND_TYPE {} provided. Only allowed values are 'docker' or 'kubernetes'.", backend_type_key); }
    };

    if args.command {
        command::run(args.group.unwrap(), backend_type);
    } else if args.health {
        health::run();
    } else {
        entrypoint::run(backend_type);
    }
}
