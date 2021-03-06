#[tokio::main]
#[cfg(unix)]
#[cfg(feature="runtime")]
async fn main() {
    tokio::spawn(async {
        println!("Listening on {:?}", cosi::consts::SOCKET_RUNTIME.to_owned());

        let service = cosi::machinery::runtime::v1alpha1::RuntimeService::default();

        service.serve(cosi::consts::SOCKET_RUNTIME.to_owned()).await
    });

    cosi::unix::handle_signals(
        || {
            std::process::exit(0);
        },
        || {
            std::process::exit(0);
        },
        || {
            std::process::exit(0);
        },
        || false,
    )
    .await;
}
