use std::{
    env,
    io::{self, BufRead},
    path::Path,
    process::Command,
    thread,
    time::Duration,
};

use console::Term;
use inotify::{Inotify, WatchMask};

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let term = Term::stdout();

    let mut args = env::args();
    // Move past the command
    args.next();

    match args.next() {
        None => {
            println!("This command expects something to execute being passed.");
            println!("Like this:");
            println!("\twatch-changes cargo build");
        }
        Some(command) => {
            let mut command_args: Vec<String> = Vec::new();
            while let Some(cmd_arg) = args.next() {
                command_args.push(cmd_arg);
            }

            let mut inotify =
                Inotify::init().expect("Failed to initialize inotify. Is it actually available?");

            loop {
                // Read in all lines from stdin
                match stdin.lock().read_line(&mut buffer) {
                    Ok(bytes) => {
                        // 0 bytes read means stdin is empty (fully read)
                        if bytes == 0 {
                            break;
                        }
                    }

                    _ => panic!("Failed to read from STDIN"),
                }
            }

            buffer.split("\n").for_each(|file| {
                let path = Path::new(file);
                // The buffer will contain a final new line, skip that
                if path.exists() {
                    inotify
                        .add_watch(path.to_str().unwrap(), WatchMask::MODIFY)
                        .expect(
                            format!("Failed to set up a file watch for {}", path.display())
                                .as_str(),
                        );
                }
            });

            let mut buffer = [0; 1024];

            loop {
                let _events = inotify
                    .read_events_blocking(&mut buffer)
                    .expect("Error while reading events");

                term.clear_screen()
                    .expect("Failed to clear the terminal screen");
                Command::new(&command)
                    .args(&command_args)
                    .spawn()
                    .expect("Failed to execute process");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
