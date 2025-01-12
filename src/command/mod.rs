use std::{process, thread, time::Duration};

use crate::{docker, kubernetes, BackendType};

/// Run the command to start a group
pub fn run(group: String, backend_type: BackendType) {
    info!(target: "lazymc-docker-proxy::command", "Received command to start group: {}", group);
    // Set a handler for SIGTERM
    let cloned_group = group.clone();
    let cloned_backend_type = backend_type.clone();
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::command", "Received SIGTERM, stopping server...");
        match cloned_backend_type {
            BackendType::Docker => {docker::stop(cloned_group.clone())}
            BackendType::Kubernetes => {kubernetes::stop(cloned_group.clone())}
        }
        process::exit(0);
    }).unwrap();

    // Start the command
    match backend_type {
        BackendType::Docker => {docker::start(group.clone())},
        BackendType::Kubernetes => {kubernetes::start(group.clone())},
    }

    // Wait for SIGTERM
    loop {
        trace!(target: "lazymc-docker-proxy::command", "Waiting for SIGTERM...");
        thread::sleep(Duration::from_secs(1));
    }
}
