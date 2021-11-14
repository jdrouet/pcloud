# pCloud cli

⚠️ This is not an official client ⚠️

## Installation

```bash
# From crates.io
cargo install pcloud-cli
```

## Using `pcloud-cli`

To be able to connect to the pcloud server, you need to create the following configuration file.

```bash
$ cat ~/.config/pcloud.json
{
        "credentials": {
                "username": "your-email-address",
                "password": "your-password"
        },
        "region": {
                "name": "eu|us"
        }
}
```

You can then use `pcloud-cli`


```bash
$ pcloud-cli --help
pcloud-cli 0.2.1

Jeremie Drouet <jeremie.drouet@gmail.com>

CLI for pcloud

USAGE:
    pcloud-cli [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config <CONFIG>    Path to load the configuration file. Default to ~/.config/pcloud.json.
                             If not found, loading from environment.
    -h, --help               Print help information
    -v, --verbose
    -V, --version            Print version information

SUBCOMMANDS:
    file      File related sub command
    folder    Folder related sub command
    help      Print this message or the help of the given subcommand(s)
```
