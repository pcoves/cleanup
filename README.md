# Cleanup

## Help

```
USAGE:
    cleanup [FLAGS] [OPTIONS]

FLAGS:
        --apply      Apply command, defaults to false
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --profile <profile>     [default: default]
    -r, --region <region>       [default: EuWest1]
```

## Native

```bash
git clone https://gitlab.com/pcoves/cleanup.git
cd cleanup
cargo build
```

### Dry run

```bash
cargo run
```

Output the list of snapshots tied to no AMI.

### :warning: Apply :warning:

```bash
cargo run -- --apply
```

Effectively deletes unused snapshots.

## Docker

```
docker run --rm -v $HOME/.aws/credentials:/root/.aws/credentials:ro registry.gitlab.com/pcoves/cleanup:latest
```
