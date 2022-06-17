# watch-changes

_Very_ basic file watcher thingy, based on [inotify](https://docs.rs/inotify/latest/inotify/index.html), meaning it will only work on linux.

## How to use this

This command takes path(s) to follow from STDIN and executes whatever is passed as command line args.
For example `git ls-files | watch-changes cargo build` will set up file watches for everything that git tracks
on a repo and execute `cargo build` if any file changed.
