use std::{
    env,
    io::{self, BufRead},
    path::Path,
    process::Command,
    sync::mpsc::channel,
    thread,
    time::Duration,
};

use console::{Key, Term};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

static ENTER_HINT: &str = "You can trigger the command manually by pushing the <ENTER> key";

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    // The channel will get the events

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
            eprintln!("{}", ENTER_HINT);
            let mut command_args: Vec<String> = Vec::new();
            while let Some(cmd_arg) = args.next() {
                command_args.push(cmd_arg);
            }

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

            let (command_sender, command_receiver) = channel();

            let commandt = thread::spawn(move || {
                let term = Term::stderr();
                loop {
                    command_receiver.recv().unwrap();
                    term.clear_screen()
                        .expect("Failed to clear the terminal screen");
                    term.write_line(ENTER_HINT).unwrap();
                    term.clear_line().unwrap();
                    let output = Command::new(&command)
                        .args(&command_args)
                        .output()
                        .expect("Failed to execute process");

                    // Need to write line by line
                    // the line write offsets will go out of sync with the terminal
                    // and this needs to be fixed per line
                    String::from_utf8_lossy(&output.stderr)
                        .split("\n")
                        .for_each(|line| {
                            term.write_line(line).unwrap();
                            // This seems to fix the line offsets screwing up
                            term.clear_line().unwrap();
                        });
                }
            });

            let enter_command_sender = command_sender.clone();
            let entert = thread::spawn(move || {
                let command_sender = enter_command_sender;
                let term = Term::stdout();
                loop {
                    let key = term.read_key().unwrap();
                    if key == Key::Enter {
                        command_sender.send(()).unwrap();
                    }
                }
            });

            let loop_command_sender = command_sender.clone();
            let loopt = thread::spawn(move || {
                let command_sender = loop_command_sender;
                let (sender, receiver) = channel();
                // The watcher will deliver debounced events once every second, should be plenty
                let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();

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
                                command_sender.send(()).unwrap();
                            }
                            // Path no longer exists, no need to watch anymore
                            DebouncedEvent::Remove(path) => {
                                watcher.unwatch(path).unwrap();
                                command_sender.send(()).unwrap();
                            }
                            // File was written, do X
                            DebouncedEvent::Write(_path) => {
                                command_sender.send(()).unwrap();
                            }
                            // Ignore all other events for now
                            _ => (),
                        },
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
            });

            loopt.join().unwrap();
            commandt.join().unwrap();
            entert.join().unwrap();
        }
    }
}
