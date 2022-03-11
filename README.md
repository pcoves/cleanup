# Cleanup

 [[_TOC_]]

## Usage

### Help

```
Search for AMI's or orphan Snapshots to delete

USAGE:
    cleanup [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --apply                  Delete AMIs/Snapshots
    -e, --endpoint <ENDPOINT>    Custom endpoint for testing purpose
    -h, --help                   Print help information
    -r, --region <REGION>        [default: eu-west-1]
    -V, --version                Print version information

SUBCOMMANDS:
    ami         Search for unused AMIs to delete
    help        Print this message or the help of the given subcommand(s)
    snapshot    Search for orphaned snaphots to delete
    volume      Search for orphaned volumes to delete
```

### Volume

```
Search for orphaned volumes to delete

USAGE:
    cleanup volume [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    Filter by Tag:Name
```

### Snapshot

```
Search for orphaned snaphots to delete

USAGE:
    cleanup snapshot [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    Filter by Tag:Name
```

### AMI

```
Search for unused AMIs to delete

USAGE:
    cleanup ami [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    Filter by AMI name/prefix,
    -t, --tag <TAG>      Filter by Tag:Name

SUBCOMMANDS:
    before    AMI's expiration date
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
