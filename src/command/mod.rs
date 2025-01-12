use std::{process, thread, time::Duration};

use crate::{docker, kubernetes};

/// Run the command to start a group
pub fn run(group: String) {
    info!(target: "lazymc-docker-proxy::command", "Received command to start group: {}", group);
    // Set a handler for SIGTERM
    let cloned_group = group.clone();
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::command", "Received SIGTERM, stopping server...");
        docker::stop(cloned_group.clone());
        kubernetes::stop(cloned_group.clone());
        process::exit(0);
    }).unwrap();

    // Start the command
    // docker::start(group.clone());
    kubernetes::start(group.clone());

    // Wait for SIGTERM
    loop {
        trace!(target: "lazymc-docker-proxy::command", "Waiting for SIGTERM...");
        thread::sleep(Duration::from_secs(1));
    }
}
