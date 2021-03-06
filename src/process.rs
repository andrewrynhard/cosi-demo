use std::io::{BufRead, BufReader, Write};
use std::process::Command;

pub fn monitor(executable: String, socket: String) -> Result<(), std::io::Error> {
    loop {
        let mut cmd = match Command::new(executable.clone())
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => return Err(err),
        };

        let stdout = cmd.stdout.take().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        let stdin = cmd.stdin.as_mut().unwrap();
        stdin.write_all(socket.as_bytes()).unwrap();

        let prefix = format!("<{}>", executable);

        std::thread::spawn(move || {
            for line in stdout_lines {
                match line {
                    Ok(line) => {
                        println!("{}: {}", prefix, line)
                    }
                    Err(err) => println!("{}: {}", prefix, err),
                }
            }
        });

        match cmd.wait() {
            Ok(status) => match status.code() {
                Some(code) => println!("Restarting {:?}, exit code: {}", executable, code),
                None => println!("Restarting {:?}, no exit code", executable),
            },
            Err(err) => println!("Restarting {:?}: {}", executable, err),
        };

        std::thread::sleep(std::time::Duration::from_millis(750));
    }
}
