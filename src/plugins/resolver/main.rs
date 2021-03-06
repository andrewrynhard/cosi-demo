use cosi::{
    machinery::plugin,
    spec::{Metadata, Resource, ResourceDefinition},
};
use std::io::{stdin, Read};

pub static NAME: &str = "resolver";

// Resource.
pub static API: &str = "cosi.dev";
pub static VERSION: &str = "v1alpha1";
pub static KIND: &str = "Resolver";
pub static NAMESPACE: &str = "core";

#[tokio::main]
#[cfg(unix)]
async fn main() {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).unwrap();

    let socket = buffer.as_str().to_owned();

    let r = plugin::register(
        socket,
        NAME.to_owned(),
        vec![Resource {
            metadata: Some(Metadata {
                api: API.to_owned(),
                version: VERSION.to_owned(),
                kind: KIND.to_owned(),
                namespace: NAMESPACE.to_owned(),
            }),
            payload: Some(cosi::spec::resource::Payload::Definition(
                ResourceDefinition {
                    dependencies: vec![],
                },
            )),
        }],
    )
    .await;

    match r {
        Ok(_) => println!("Registered {}", NAME),
        Err(err) => match err.code() {
            tonic::Code::AlreadyExists => println!("Already registered"),
            _ => std::process::exit(1),
        },
    }

    cosi::unix::handle_signals(
        || {
            std::process::exit(0);
        },
        || {
            std::process::exit(0);
        },
        || {},
    )
    .await;
}
