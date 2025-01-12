mod config;
use config::Config;
use log::Level;
use regex::Regex;
use std::{io::{BufRead, BufReader}, process::{self, exit}};

use crate::{docker, health::{self}, kubernetes, BackendType};

/// Entrypoint for the application
pub fn run(backend_type: BackendType) {
    // Ensure all server containers are stopped before starting
    info!(target: "lazymc-docker-proxy::entrypoint", "Ensuring all server containers are stopped...");
    match backend_type {
        BackendType::Docker => {docker::stop_all_containers();}
        BackendType::Kubernetes => {kubernetes::stop_all_containers();}
    }

    let labels_list = match backend_type {
        BackendType::Docker => { docker::get_container_labels() }
        BackendType::Kubernetes => { kubernetes::get_container_labels() }
    };
    let mut configs: Vec<Config> = Vec::new();
    let mut children: Vec<process::Child> = Vec::new();

    for label in labels_list {
        configs.push(Config::from_container_labels(label));
    }

    if configs.is_empty() {
        configs.push(Config::from_env(backend_type.clone()));
    }

    for config in configs {
        let group: String = config.group().into();

        info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process for group: {}...", group.clone());
        let mut child: process::Child = config.start_command(backend_type.clone())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdout = child.stdout.take();
        let group_clone = group.clone();
        let backend_type_clone = backend_type.clone();
        std::thread::spawn(move || {
            let stdout_reader = BufReader::new(stdout.take().unwrap());
            for line in stdout_reader.lines() {
                wrap_log(&group_clone, line, &backend_type_clone);
            }
        });

        let mut stderr = child.stderr.take();
        let backend_type_clone = backend_type.clone();
        std::thread::spawn(move || {
            let stderr_reader = BufReader::new(stderr.take().unwrap());
            for line in stderr_reader.lines() {
                wrap_log(&group.clone(), line, &backend_type_clone)
            }
        });

        children.push(child);
    }

    // If this app receives a signal, stop all server containers
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::entrypoint", "Received exit signal. Stopping all server containers...");
        match backend_type {
            BackendType::Docker => { docker::stop_all_containers(); }
            BackendType::Kubernetes => { kubernetes::stop_all_containers(); }
        }
        exit(0);
    }).unwrap();

    // Set the health status to healthy
    health::healthy();

    // wait indefinitely
    loop {
        std::thread::park();
    }


}

/// Wrap log messages from child processes
fn wrap_log(group: &String, line: Result<String, std::io::Error>, backend_type: &BackendType) {
    if let Ok(line) = line {
        let regex: Regex = Regex::new(r"(?P<level>[A-Z]+)\s+(?P<target>[a-zA-Z0-9:_-]+)\s+>\s+(?P<message>.+)$").unwrap();
        if let Some(captures) = regex.captures(&line) {
            let level: Level = captures.name("level").unwrap().as_str().parse().unwrap();
            let target = captures.name("target").unwrap().as_str();
            let message = captures.name("message").unwrap().as_str();

            let wrapped_target = &format!("{}::{}", group, target);
            let log_message = format!("{}", message);
            log!(target: wrapped_target, level, "{}", log_message);
            handle_log(group, &level, &log_message, &backend_type);
        } else {
            print!("{}", line);
        }
    }
}

/// Handle log messages that require special attention
fn handle_log(group: &String, level: &Level, message: &String, backend_type: &BackendType) {
    match (level, message.as_str()) {
        (Level::Warn, "Failed to stop server, no more suitable stopping method to use") => {
            warn!(target: "lazymc-docker-proxy::entrypoint", "Unexpected server state detected, force stopping {} server container...", group.clone());
            match backend_type {
                BackendType::Docker => { docker::stop(group.clone()) }
                BackendType::Kubernetes => { kubernetes::stop(group.clone()) }
            }
            info!(target: "lazymc-docker-proxy::entrypoint", "{} server container forcefully stopped", group.clone());
        }
        _ => {}
    }
}
