# watch-changes

_Very_ basic file watcher thingy, based on [notify](https://docs.rs/notify/latest/notify/index.html).

## How to use this

This command takes path(s) to follow from STDIN and executes whatever is passed as command line args.
For example `git ls-files | watch-changes cargo build` will set up file watches for everything that git tracks
on a repo and execute `cargo build` if any file changed.
