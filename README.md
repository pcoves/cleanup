# Cleanup

 [[_TOC_]]

```
cleanup 0.2.0
Search for AMI's or orphan Snapshots to delete

USAGE:
    cleanup [OPTIONS] [SUBCOMMAND]

OPTIONS:
        --apply              Delete AMIs/Snapshots
    -h, --help               Print help information
    -n, --name <NAME>        AMIs name (or prefix if ends with a *)
    -r, --region <REGION>    [default: eu-west-1]
    -V, --version            Print version information

SUBCOMMANDS:
    before    Keep AMIs based on their age
    help      Print this message or the help of the given subcommand(s)
    keep      How many AMIs to keep
```

## Build

```
cargo build
```

## Install

```
cargo install --path .
```
