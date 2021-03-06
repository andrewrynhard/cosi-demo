use cosi::{
    consts::SOCKET_RUNTIME,
    spec::{self, ApplyResponse},
    ResourceInstance,
};
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("failed to read the resource");

    let resource: ResourceInstance =
        serde_yaml::from_str(&contents).expect("failed to deserialize YAML");

    let mut client = cosi::machinery::runtime::client::connect(SOCKET_RUNTIME.to_owned())
        .await
        .unwrap();

    let json = serde_json::to_string(&resource.spec).unwrap();
    let spec = json.as_bytes();

    let request = tonic::Request::new(spec::Resource {
        metadata: Some(spec::Metadata {
            api: resource.api,
            version: resource.version,
            kind: resource.kind,
            namespace: resource.namespace,
        }),
        payload: Some(spec::resource::Payload::Instance(spec::ResourceInstance {
            id: resource.id,
            spec: Some(spec.to_owned()),
        })),
    });

    println!("Applying {}", filename);

    let response = client.apply(request).await.unwrap();

    let r: ApplyResponse = response.into_inner();

    println!("{:?}", r);

    Ok(())
}
