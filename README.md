# Cleanup

 [[_TOC_]]

## Usage

### Help

```
Search for Image's or orphan Snapshots/Volume to delete

USAGE:
    cleanup [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --apply                  Delete Images/Snapshots
    -h, --help                   Print help information
    -p, --profile <PROFILE>      [default: default]
    -r, --region <REGION>        [default: eu-west-1]
    -V, --version                Print version information

SUBCOMMANDS:
    help        Print this message or the help of the given subcommand(s)
    image       Search for unused images to delete
    snapshot    Search for orphaned snaphots to delete
    volume      Search for orphaned volumes to delete
```

## Build

```
cargo build
```

## Install

```
cargo install --path .
```
