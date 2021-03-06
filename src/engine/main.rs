use nix::sys::wait::waitpid;
use nix::sys::wait::WaitPidFlag;
use nix::unistd::Pid;

#[tokio::main]
#[cfg(unix)]
async fn main() {
    tokio::spawn(async {
        println!("Listening on {:?}", cosi::consts::SOCKET_ENGINE.to_owned());

        let service = cosi::machinery::engine::v1alpha1::EngineService::default();

        service.serve(cosi::consts::SOCKET_ENGINE.to_owned()).await
    });

    tokio::spawn(async {
        match cosi::machinery::runtime::load(cosi::consts::SOCKET_ENGINE.to_owned()).await {
            Ok(_) => println!("Runtime loaded"),
            Err(err) => println!("Failed to load runtime: {:?}", err),
        };

        match cosi::machinery::plugin::load(cosi::consts::SOCKET_ENGINE.to_owned()).await {
            Ok(n) => println!("{} Plugin(s) loaded", n),
            Err(err) => println!("Failed to load plugins: {:?}", err),
        };
    });

    cosi::unix::handle_signals(
        || {
            println!("Ignoring SIGHUP");
        },
        || {
            println!("Ignoring SIGINT");
        },
        || {
            // https://man7.org/linux/man-pages/man2/waitpid.2.html
            let status = waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG));
            if let Err(err) = status {
                println!("Failed to reap zombie: {:?}", err);
            }
        },
    )
    .await;
}
