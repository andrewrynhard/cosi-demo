mod common;

use common::setup;
use cosi::{
    machinery::plugin,
    spec::{resource, Metadata, Resource, ResourceDefinition, ResourceInstance},
};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;

#[test]
fn engine() {
    let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();

    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let dir = setup().unwrap();

        let engine_path = dir.join("engine.sock");
        let engine_socket = engine_path.into_os_string().into_string().unwrap();

        let runtime_path = dir.join("runtime.sock");
        let runtime_socket = runtime_path.into_os_string().into_string().unwrap();

        let e1 = engine_socket.clone();

        tokio::spawn(async {
            println!("Engine: {}", e1);

            let service = cosi::machinery::engine::v1alpha1::EngineService::default();
            service.serve(e1).await;

            panic!("expected engine server to not fail")
        });

        let r1 = runtime_socket.clone();

        tokio::spawn(async {
            println!("Runtime: {}", r1);

            let service = cosi::machinery::runtime::v1alpha1::RuntimeService::default();
            service.serve(r1).await;

            panic!("expected runtime server to not fail")
        });

        tokio::spawn(async {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let result = plugin::register(
                engine_socket,
                String::from("test"),
                vec![Resource {
                    metadata: Some(Metadata {
                        api: String::from("cosi.dev"),
                        version: String::from("v1alpha1"),
                        kind: String::from("Test"),
                        namespace: String::from("system"),
                    }),
                    payload: Some(cosi::spec::resource::Payload::Definition(
                        ResourceDefinition {
                            dependencies: vec![],
                        },
                    )),
                }],
            )
            .await;

            assert!(result.is_ok())
        });

        tokio::spawn(async move {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let mut client = cosi::machinery::runtime::client::connect(runtime_socket)
                .await
                .unwrap();

            let request = tonic::Request::new(Resource {
                metadata: Some(Metadata {
                    api: String::from("cosi.dev"),
                    version: String::from("v1alpha1"),
                    kind: String::from("Test"),
                    namespace: String::from("system"),
                }),
                payload: Some(resource::Payload::Instance(ResourceInstance {
                    id: String::from("test"),
                    spec: Some(r#"{"test": true}"#.as_bytes().to_owned()),
                })),
            });

            let result = client.apply(request).await;
            assert!(result.is_ok());

            println!("{:?}", result.unwrap());

            assert!(tx.send(()).is_ok());
        });
    });

    rx.recv().unwrap();
}
