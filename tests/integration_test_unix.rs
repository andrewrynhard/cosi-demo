use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult};

#[tokio::test]
async fn zombie_reap() {
    unsafe {
        match fork() {
            Ok(ForkResult::Child) => {
                std::process::exit(0);
            }
            Ok(ForkResult::Parent { child, .. }) => {
                cosi::unix::handle_signals(
                    || {
                        std::process::exit(0);
                    },
                    || {
                        std::process::exit(0);
                    },
                    || {
                        let processes = procfs::process::all_processes().unwrap();

                        for process in &processes {
                            if process.stat.pid == child.as_raw() {
                                let stat = process.stat().unwrap();
                                let state = stat.state().unwrap();
                                assert_eq!(state, procfs::process::ProcState::Zombie);
                            }
                        }

                        let status = waitpid(child, None);

                        match status {
                            Ok(s) => {
                                let pid = s.pid().unwrap();
                                assert_eq!(child, pid);
                                std::process::exit(0);
                            }
                            Err(err) => {
                                panic!("{}", err);
                            }
                        }
                    },
                )
                .await;
            }
            Err(err) => panic!("{}", err),
        };
    }
}
