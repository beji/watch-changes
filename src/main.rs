use std::{
    env,
    io::{self, BufRead},
    path::Path,
    process::Command,
    sync::mpsc::channel,
    time::Duration,
};

use console::Term;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let term = Term::stdout();
    // The channel will get the events
    let (sender, receiver) = channel();

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

            // The watcher will deliver debounced events once every second, should be plenty
            let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();

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
                    watcher
                        .watch(path.to_str().unwrap(), RecursiveMode::NonRecursive)
                        .expect(
                            format!("Failed to set up a file watch for {}", path.display())
                                .as_str(),
                        );
                }
            });

            loop {
                match receiver.recv() {
                    Ok(event) => match event {
                        // Watch the new path, stop watching the old
                        DebouncedEvent::Rename(old, new) => {
                            watcher.unwatch(old).unwrap();
                            watcher.watch(new, RecursiveMode::NonRecursive).unwrap();
                        }
                        // Path no longer exists, no need to watch anymore
                        DebouncedEvent::Remove(path) => {
                            watcher.unwatch(path).unwrap();
                        }
                        // File was written, do X
                        DebouncedEvent::Write(_path) => {
                            term.clear_screen()
                                .expect("Failed to clear the terminal screen");
                            Command::new(&command)
                                .args(&command_args)
                                .spawn()
                                .expect("Failed to execute process");
                        }
                        // Ignore all other events for now
                        _ => (),
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        }
    }
}
