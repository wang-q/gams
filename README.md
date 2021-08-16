# garr

![Publish](https://github.com/wang-q/garr/workflows/Publish/badge.svg)
![Build](https://github.com/wang-q/garr/workflows/Build/badge.svg)

Genome Analyst with Rust and Redis

## INSTALL

Current release: 0.0.1

```shell script
cargo install --force --offline --path .

```


## SYNOPSIS

```
$ garr help
garr 0.0.1
wang-q <wang-q@outlook.com>
Genome Analyst with Rust and Redis

USAGE:
    garr [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    env       Create a .env file
    gen       Generate the database from (gzipped) fasta files
    help      Prints this message or the help of the given subcommand(s)
    status    Test Redis config and connection

```

## RUNTIME DEPENDENCIES

* Command line tools managed by `Linuxbrew`

```shell script
brew install redis
# scoop install redis5          # for redis-cli
brew install parallel wget pigz
brew install datamash mlr

brew tap wang-q/tap
brew install wang-q/tap/tsv-utils wang-q/tap/intspan

```

## EXAMPLES

```shell script
REDIS_TLS=true REDIS_PASSWORD='mYpa$$' garr env -o stdout

garr env

garr status test

garr sliding tests/S288c/genome.fa.gz

```

## AUTHOR

Qiang Wang <wang-q@outlook.com>

## LICENSE

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2021.
