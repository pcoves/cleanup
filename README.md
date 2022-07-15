# Cleanup

## Build

```
cargo build
```

## Install

```
cargo install --path .
```

### Help

```
❯ cargo run -- -h
cleanup 0.5.0
Pablo COVES <pablo.coves@protonmail.com>
Search for Image's or orphan Snapshots/Volume to delete

USAGE:
    cleanup [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help                 Print help information
    -p, --profile <PROFILE>    [default: default]
    -r, --region <REGION>      [default: eu-west-1]
    -V, --version              Print version information

SUBCOMMANDS:
    help        Print this message or the help of the given subcommand(s)
    image       Search for unused images to delete
    read        Read previously generated resource list to delete
    snapshot    Search for orphaned snaphots to delete
    volume      Search for orphaned volumes to delete

```
