# Sydney

[![crates.io](https://img.shields.io/crates/v/sydney.svg)](https://crates.io/crates/sydney)
[![github.com](https://github.com/gemrest/sydney/actions/workflows/rust.yaml/badge.svg?branch=main)](https://github.com/gemrest/sydney/actions/workflows/rust.yaml)

Sydney is a Vim-like, command-line Gemini client.

[![Video of Sydney in action](https://host.fuwn.me/m3q9cny6pr5f.png)](https://i.imgur.com/l2gw8wp.mp4)

Sydney has a beautiful, intuitive, and powerful command-line interface;
including:

- Vim-like keybindings
- Vim-like commands
- Intuitive link handling
- Understandable errors
- Customizable UI

## Usage

### Installation

```shell
cargo install sydney --force
```

### Help

```shell
usage: syndey [option, capsule_uri]
Options:
    --version, -v    show version text
    --help, -h       show help text

Sample invocations:
    syndey gemini://fuwn.me/
    syndey fuwn.me
    syndey --help

Report bugs to https://github.com/gemrest/sydney/issues

```

## License

This project is licensed with the [GNU General Public License v3.0](https://github.com/gemrest/sydney/blob/main/LICENSE).
