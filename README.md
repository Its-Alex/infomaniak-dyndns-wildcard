# Infomaniak-dyndns-wildcard

This tool is used to auto update IP on a wildcard dns record for infomaniak
since their dyndns solution don't support wildcard dns records.

This project support only one dns record for now since it's mostly used for
wildcard. If you're not updating a wildcard DNS record you should see
[Infomaniak Dyndns](https://www.infomaniak.com/fr/domaines/dyndns).

This is my first project in Rust. It is intended as a way to learn the language,
so I am fully aware that the code may be lacking. Please don't hesitate to make
a PR to improve it!

## Requirements

- [mise](https://mise.jdx.dev/)
- [docker](https://www.docker.com/) (if you want to build container)

You must run theses commands the first time:

```sh
$ mise trust && mise install
```

## Getting started

This tool can be used as a docker container or directly using rust.

### How to use docker container

You can use it in a compose file with the following example:

```docker-compose
services:
  infomaniak-dyndns-wildcard:
    image: itsalex/infomaniak-dyndns-wildcard:latest
    environment:
      - INFOMANIAK_DYNDNS_WILDCARD_INFOMANIAK_API_TOKEN=<your-informaniak-token>
      - INFOMANIAK_DYNDNS_WILDCARD_TIME_BETWEEN_UPDATES_IN_SECONDS=<time-between-update-in-seconds>
      - INFOMANIAK_DYNDNS_WILDCARD_DNS_ZONE_ID=<your-dns-zone>
      - INFOMANIAK_DYNDNS_WILDCARD_RECORD_NAME=<your-dns-record> # In our case certainly a "*" (wildcard) or "*.example"
```

## How to hack

First, you should set all environment variables beginning with
`INFOMANIAK_DYNDNS_WILDCARD_INFOMANIAK` in [.envrc](./.envrc) in your shell.

When this is done, you can run the app locally using:

```sh
$ cargo run
   Compiling infomaniak-dyndns-wildcard v0.1.0 (/home/alex/Documents/infomaniak-dyndns-wildcard-domain)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.81s
     Running `target/debug/infomaniak-dyndns-wildcard`
Public IP: 176.130.154.147
...
```

#### How to lint

To lint the project you can use [clippy](https://github.com/rust-lang/rust-clippy):

```sh
$ rustup component add clippy
$ cargo clippy --all-targets --all-features -- -D warnings
```

#### How to format

To format the project you can use [rustfmt](https://github.com/rust-lang/rustfmt):

```sh
$ rustup component add rustfmt
$ cargo fmt --all
```

# License

[MIT](./LICENSE)