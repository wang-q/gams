# `gars`

![Publish](https://github.com/wang-q/gars/workflows/Publish/badge.svg)
![Build](https://github.com/wang-q/gars/workflows/Build/badge.svg)

`gars` - **G**enome **A**nalyst with **R**ust and redi**S**

## INSTALL

Current release: 0.1.0

```shell
cargo install --force --offline --path .

```


## SYNOPSIS

```text
$ gars help
gars 0.1.1-alpha.0
wang-q <wang-q@outlook.com>
Genome Analyst with Rust and Redis

USAGE:
    gars [SUBCOMMAND]

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

```shell
brew install redis

brew install parallel wget aria2 pigz
brew install datamash miller

brew tap wang-q/tap
brew install wang-q/tap/tsv-utils wang-q/tap/intspan wang-q/tap/faops

```

* R (4.1) packages

```shell
# R packages
parallel -j 1 -k --line-buffer '
    Rscript -e '\'' if (!requireNamespace("{}", quietly = TRUE)) { install.packages("{}", repos="https://mirrors.tuna.tsinghua.edu.cn/CRAN") } '\''
    ' ::: \
        getopt \
        extrafont ggplot2 gridExtra \
        tidyverse

```

* Querying tools

```shell

# Redis GUI
# winget install qishibo.AnotherRedisDesktopManager

# Clickhouse
export LTS=21.8.4.51
curl -LO https://repo.clickhouse.tech/tgz/lts/clickhouse-common-static-${LTS}.tgz

tar -xzvf clickhouse-common-static-${LTS}.tgz
sudo bash clickhouse-common-static-${LTS}/install/doinst.sh

# Clickhouse GUI
git clone https://github.com/VKCOM/lighthouse
browser lighthouse/index.html

# textql as an alternative
brew install textql

```

## EXAMPLES

```shell
REDIS_TLS=true REDIS_PASSWORD='mYpa$$' gars env -o stdout

gars env

gars status test

```

## AUTHOR

Qiang Wang <wang-q@outlook.com>

## LICENSE

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2021.
