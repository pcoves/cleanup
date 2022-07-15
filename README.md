# Cleanup

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

## Build

```
cargo build
```

## Install

```
cargo install --path .
```

## Usage

### images

```
❯ cargo run -- image -h
cleanup-image
Search for unused images to delete

USAGE:
    cleanup image [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --apply                            Effectively deletes images
    -h, --help                             Print help information
    -n, --names <NAMES>                    Filter by image name/prefix,
    -N, --exclude-names <EXCLUDE_NAMES>    Exclude images with matching names
    -o, --output <OUTPUT>                  Save result for later deletion
    -t, --tags <TAGS>                      Filter by Tags
    -T, --exclude-tags <EXCLUDE_TAGS>      Exclude images with matching tags

SUBCOMMANDS:
    before    Image's expiration date
    help      Print this message or the help of the given subcommand(s)
    keep      How many images to keep
```

#### Filters grammar

The `-n/--names` and `-t/--tags` filters are passed *as-is* to the `AWS` API and are treated on the server side using `AWS` [regex engine](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Filtering.html#Filtering_Resources_CLI)
This means n asterisk `*` matches zero or more characters, and a question mark `?` matches zero or one character.

On the other hand, the `-N/--exclude-names` and `-T/--exclude-tags` are treated using Rust's [regex](https://docs.rs/regex/latest/regex/) engine.
The [syntax](https://docs.rs/regex/latest/regex/#syntax) is somewhat different between the two.

```
❯ RUST_LOG=cleanup::aws=info cargo run -- -p ottobock-dev image -t "base-ami-owner-id 810084356657" -T "base-ami-id ami-07360a32a73af9217" -n anato-brace-webapp-zeta* -N anato-brace-webapp-zeta-2022-04-26_072626_157 -o /tmp/images.json keep 2
[2022-07-15T14:04:37Z INFO  cleanup::aws::image] Found 15 matching images
[2022-07-15T14:04:38Z INFO  cleanup::aws::image] 15 of them are unused
[2022-07-15T14:04:38Z INFO  cleanup::aws::image] Kept 14 after excluding by name
[2022-07-15T14:04:38Z INFO  cleanup::aws::image] Kept 5 after excluding by tag
[2022-07-15T14:04:38Z INFO  cleanup::aws::image] Will delete 3 images and associated data
```
