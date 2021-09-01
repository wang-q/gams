# `garr`

![Publish](https://github.com/wang-q/garr/workflows/Publish/badge.svg)
![Build](https://github.com/wang-q/garr/workflows/Build/badge.svg)

Genome Analyst with Rust and Redis

## INSTALL

Current release: 0.1.0

```shell script
cargo install --force --offline --path .

```


## SYNOPSIS

```
$ garr help
garr 0.1.1-alpha.0
wang-q <wang-q@outlook.com>
Genome Analyst with Rust and Redis

USAGE:
    garr [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    env        Create a .env file
    gen        Generate the database from (gzipped) fasta files
    help       Prints this message or the help of the given subcommand(s)
    pos        Add range files to positions
    range      Add ranges
    rsw        Sliding windows around a range
    sliding    Sliding windows along a chromosome
    status     Test Redis config and connection
    tsv        Exports Redis hashes to a tsv file
    wave       Add peaks of GC-waves

```

## RUNTIME DEPENDENCIES

* Command line tools managed by `Linuxbrew`

```shell script
brew install redis

brew install parallel wget pigz
brew install datamash miller

brew tap wang-q/tap
brew install wang-q/tap/tsv-utils wang-q/tap/intspan

```

* R (4.1) packages

```shell script
# R packages
parallel -j 1 -k --line-buffer '
    Rscript -e '\'' if (!requireNamespace("{}", quietly = TRUE)) { install.packages("{}", repos="https://mirrors.tuna.tsinghua.edu.cn/CRAN") } '\''
    ' ::: \
        getopt \
        extrafont ggplot2 gridExtra \
        tidyverse

```

* Other tools

```shell script

# clickhouse
curl -LO https://github.com/ClickHouse/ClickHouse/releases/download/v21.8.4.51-lts/clickhouse-common-static-21.8.4.51.tgz
tar xvfz clickhouse-common-static-21.8.4.51.tgz
sudo bash ./clickhouse-common-static-21.8.4.51/install/doinst.sh

# winget install qishibo.AnotherRedisDesktopManager

```

## EXAMPLES

```shell script
REDIS_TLS=true REDIS_PASSWORD='mYpa$$' garr env -o stdout

garr env

garr status test

```

## AUTHOR

Qiang Wang <wang-q@outlook.com>

## LICENSE

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2021.
