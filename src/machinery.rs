pub mod engine {
    pub mod client {
        use crate::spec::engine_client::EngineClient;
        use std::convert::TryFrom;
        use tokio::net::UnixStream;
        use tonic::transport::{Endpoint, Uri};
        use tower::service_fn;

        pub async fn connect(
            socket: String,
        ) -> Result<EngineClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(EngineClient::new(channel))
        }
    }

    pub mod v1alpha1 {
        use crate::{
            spec::{
                engine_server::{Engine, EngineServer},
                Plugin, RegisterResponse,
            },
            unix,
        };
        use std::{
            fs,
            path::Path,
            sync::{Arc, Mutex},
        };
        use tonic::transport::Server;
        use tonic::{Code, Request, Response, Status};

        #[derive(Default, Clone)]
        pub struct EngineService {
            plugins: Arc<Mutex<Vec<Plugin>>>,
        }

        impl EngineService {
            pub async fn serve(self, socket: String) {
                if Path::new(&socket).exists() {
                    fs::remove_file(&socket).expect("failed to remove socket");
                }

                let s = socket.clone();

                let handle = tokio::spawn(async move {
                    match unix::UnixIncoming::bind(s) {
                        Ok(socket) => {
                            match Server::builder()
                                .add_service(EngineServer::new(self))
                                .serve_with_incoming(socket)
                                .await
                            {
                                Ok(_) => (),
                                Err(err) => println!("{:?}", err),
                            }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                });

                handle.await.unwrap()
            }
        }

        #[tonic::async_trait]
        impl Engine for EngineService {
            async fn register(
                &self,
                request: Request<Plugin>,
            ) -> Result<Response<RegisterResponse>, Status> {
                let request_plugin = request.into_inner();

                let mutex = self.plugins.clone();
                let mut registered_plugins = mutex.lock().unwrap();

                for registered_plugin in registered_plugins.clone().into_iter() {
                    if registered_plugin.name == request_plugin.name {
                        return Err(Status::new(
                            Code::AlreadyExists,
                            format!("plugin is already registered: {:?}", request_plugin.name),
                        ));
                    }

                    for registered_resource in registered_plugin.resources {
                        for request_resource in request_plugin.resources.clone() {
                            if registered_resource == request_resource {
                                return Err(Status::new(
                                    Code::AlreadyExists,
                                    format!(
                                        "resource definition is already registered: {:?}",
                                        request_resource
                                    ),
                                ));
                            }
                        }
                    }
                }

                println!("{:?}", request_plugin);

                registered_plugins.push(request_plugin);

                Ok(Response::new(RegisterResponse {}))
            }
        }
    }
}

pub mod runtime {
    use crate::consts;

    pub async fn load(socket: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading runtime");

        std::thread::spawn(move || {
            crate::process::monitor(consts::RUNTIME.to_owned(), socket).unwrap();
        });

        Ok(())
    }

    pub mod client {
        use crate::spec::runtime_client::RuntimeClient;
        use std::convert::TryFrom;
        use tokio::net::UnixStream;
        use tonic::transport::{Endpoint, Uri};
        use tower::service_fn;

        pub async fn connect(
            socket: String,
        ) -> Result<RuntimeClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(RuntimeClient::new(channel))
        }
    }

    pub mod v1alpha1 {
        use crate::{
            spec::{
                runtime_server::Runtime, runtime_server::RuntimeServer, ApplyResponse,
                DeleteResponse, GetResponse, Resource, WatchResponse,
            },
            unix,
        };
        use futures::Stream;
        use std::{fs, path::Path, pin::Pin};
        use tonic::{transport::Server, Request, Response, Status};

        #[derive(Default, Clone, Copy)]
        pub struct RuntimeService {}

        impl RuntimeService {}

        impl RuntimeService {
            pub async fn serve(self, socket: String) {
                if Path::new(&socket).exists() {
                    fs::remove_file(&socket).expect("failed to remove socket");
                }

                let s = socket.clone();

                let handle = tokio::spawn(async move {
                    match unix::UnixIncoming::bind(s) {
                        Ok(socket) => {
                            match Server::builder()
                                .add_service(RuntimeServer::new(self))
                                .serve_with_incoming(socket)
                                .await
                            {
                                Ok(_) => (),
                                Err(err) => println!("{:?}", err),
                            }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                });

                handle.await.unwrap()
            }
        }

        #[tonic::async_trait]
        impl Runtime for RuntimeService {
            async fn apply(
                &self,
                request: Request<Resource>,
            ) -> Result<Response<ApplyResponse>, Status> {
                println!("{:?}", request.into_inner());
                Ok(Response::new(ApplyResponse {}))
            }

            async fn delete(
                &self,
                request: Request<Resource>,
            ) -> Result<Response<DeleteResponse>, Status> {
                println!("{:?}", request.into_inner());
                Ok(Response::new(DeleteResponse {}))
            }

            async fn get(
                &self,
                request: Request<Resource>,
            ) -> Result<Response<GetResponse>, Status> {
                println!("{:?}", request.into_inner());
                Ok(Response::new(GetResponse {}))
            }

            type WatchStream =
                Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send + Sync + 'static>>;

            async fn watch(
                &self,
                _request: Request<Resource>,
            ) -> Result<Response<Self::WatchStream>, Status> {
                unimplemented!()
            }
        }
    }
}

pub mod plugin {
    use crate::{
        consts,
        spec::{Plugin, RegisterResponse, Resource},
    };
    use glob::glob_with;
    use glob::MatchOptions;
    use std::env;

    pub async fn load(socket: String) -> Result<i32, Box<dyn std::error::Error>> {
        let options = MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let pattern = format!(
            "{}/*-{}-{}",
            consts::PLUGINS,
            env::consts::OS,
            env::consts::ARCH
        );

        let mut n: i32 = 0;

        for entry in glob_with(&pattern, options).unwrap() {
            if let Ok(path) = entry {
                println!("Loading plugin {:?}", path.display());

                let s = socket.clone();

                if let Ok(executable) = path.into_os_string().into_string() {
                    std::thread::spawn(move || {
                        crate::process::monitor(executable, s).unwrap();
                    });
                };

                n += 1;
            }
        }

        Ok(n)
    }

    pub async fn register(
        socket: String,
        name: String,
        resources: Vec<Resource>,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        let mut client = super::engine::client::connect(socket).await.unwrap();

        let request = tonic::Request::new(Plugin { name, resources });

        client.register(request).await
    }

    pub mod resource {
        use crate::spec::{resource::Payload, Resource};
        use serde::de;

        /// # Examples
        ///
        /// ```
        /// use cosi::spec::{Metadata, Resource};
        ///
        /// let resource = Resource {
        ///     metadata: Some(Metadata {
        ///         api: "cosi.dev".to_string(),
        ///         version: "v1alpha1".to_string(),
        ///         kind: "Mount".to_string(),
        ///         namespace: "system".to_string(),
        ///     }),
        ///     payload: Some(cosi::spec::resource::Payload::Instance(cosi::spec::ResourceInstance {
        ///         id: "test".to_string(),
        ///         spec: Some(
        ///             r#"{"type": "tmpfs", "source": "tmpfs", "target": "/tmp", "options": []}"#
        ///                 .as_bytes()
        ///                 .to_owned(),
        ///         ),
        ///     })),
        /// };
        ///
        /// let _mount: cosi::spec::Mount = cosi::machinery::plugin::resource::into_instance(&resource).unwrap();
        /// ```
        pub fn into_instance<'t, T>(resource: &'t Resource) -> Option<T>
        where
            T: de::Deserialize<'t>,
        {
            let payload = resource.payload.as_ref().unwrap();

            if let Payload::Instance(payload) = payload {
                let t: T = serde_json::from_str(
                    std::str::from_utf8(payload.spec.as_ref().unwrap().as_ref()).unwrap(),
                )
                .unwrap();
                Some(t)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_into_instance() {
        let resource = spec::Resource {
            metadata: Some(spec::Metadata {
                api: "cosi.dev".to_string(),
                version: "test".to_string(),
                kind: "Test".to_string(),
                namespace: "test".to_string(),
            }),
            payload: Some(spec::resource::Payload::Instance(spec::ResourceInstance {
                id: "test".to_string(),
                spec: Some(
                    r#"{"type": "test", "source": "test", "target": "test", "options": []}"#
                        .as_bytes()
                        .to_owned(),
                ),
            })),
        };

        let _mount: spec::Mount = machinery::plugin::resource::into_instance(&resource).unwrap();
    }
}
