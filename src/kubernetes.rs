use std::collections::HashMap;
use std::env::var;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::serde_json::json;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};
use tokio::runtime::Runtime;

pub fn start(group: String) {
    debug!(target: "lazymc-docker-proxy::kubernetes", "Scaling up StatefulSet...");

    Runtime::new().unwrap().block_on(async {
        // Erhalte eine Kubernetes-Client-Instanz.
        let client = Client::try_default().await;

        let statefulset_name = var("STATEFULSET_NAME").unwrap();
        let namespace = var("STATEFULSET_NAMESPACE").unwrap();

        info!(target: "lazymc-docker-proxy::kubernetes", "Scaling up StatefulSet {} in namespace {} to 1 replicas", statefulset_name, namespace);

        let statefulsets: Api<StatefulSet> = Api::namespaced(client.unwrap(), namespace.as_str());

        let patch = json!({
            "spec": {
                "replicas": 1,
            }
        });

        let patch_params = PatchParams::apply("scale-up-minecraft-statefulset");
        if let Err(err) = statefulsets.patch(statefulset_name.as_str(), &patch_params, &Patch::Apply(patch)).await
        {
            error!(target: "lazymc-docker-proxy::kubernetes", "Error scaling up StatefulSet {} in namespace {}: {}", statefulset_name.as_str(), namespace, err);
        }
    })
}

pub fn stop(group: String) {
    error!(target: "lazymc-docker-proxy::kubernetes", "Stopping of Kubernetes StatefulSets not yet implemented!");
}

pub fn stop_all_containers() {
    error!(target: "lazymc-docker-proxy::kubernetes", "Stopping of Kubernetes StatefulSets not yet implemented!");
}

pub(crate) fn get_container_labels() -> Vec<HashMap<std::string::String, std::string::String>> {
    error!(target: "lazymc-docker-proxy::kubernetes", "Get Container labels of Kubernetes StatefulSets not yet implemented!");
    vec![]
}