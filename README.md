# gars

[![Build](https://github.com/wang-q/gars/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/gars/actions)
[![codecov](https://codecov.io/gh/wang-q/gars/branch/main/graph/badge.svg?token=LtxYK5Fff0)](https://codecov.io/gh/wang-q/gars)
[![license](https://img.shields.io/github/license/wang-q/intspan)](https://github.com//wang-q/intspan)
[![Lines of code](https://tokei.rs/b1/github/wang-q/gars?category=code)](https://github.com//wang-q/gars)

`gars` - **G**enome **A**nalyst with **R**ust and redi**S**

## INSTALL

Current release: 0.3.1

```shell
cargo install --force --offline --path .

# test
cargo test -- --test-threads=1

# gars-stat
cargo install --force --offline --path . --features stat

# build under WSL 2
export CARGO_TARGET_DIR=/tmp
cargo build

```

## SYNOPSIS

### `gars help`

```text
gars 0.3.1
wang-q <wang-q@outlook.com>
Genome Analyst with Rust and rediS

USAGE:
    gars [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    anno       Annotate anything that contains a ctg_id and a range
    clear      Clear some parts from Redis
    env        Create a .env file
    feature    Add genomic features from range files
    fsw        Sliding windows around a feature
    gen        Generate the database from (gzipped) fasta files
    help       Print this message or the help of the given subcommand(s)
    locate     Locate the given ranges to the corresponding ctgs
    peak       Add peaks of GC-waves
    range      Add range files for counting
    sliding    Sliding windows along a chromosome
    status     Test Redis config and connection
    tsv        Export Redis hashes to a tsv file

```

### `gars-stat --help`

```text
gars-stat 0.3.1
wang-q <wang-q@outlook.com>
Build-in stats for gars

USAGE:
    gars-stat [OPTIONS] <infile> <query>

ARGS:
    <infile>    Sets the input file to use
    <query>     Query name [default: ctg]

OPTIONS:
    -h, --help                 Print help information
    -o, --outfile <outfile>    Output filename. [stdout] for screen [default: stdout]
    -V, --version              Print version information

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

* R (4.2) packages

```shell
# R packages
parallel -j 1 -k --line-buffer '
    Rscript -e '\''
        if (!requireNamespace("{}", quietly = TRUE)) {
            install.packages("{}", repos="https://mirrors.tuna.tsinghua.edu.cn/CRAN")
        }
    '\''
    ' ::: \
        getopt \
        extrafont ggplot2 gridExtra \
        tidyverse

```

* Querying tools

```shell
# Redis GUI
# winget install qishibo.AnotherRedisDesktopManager
# brew install --cask another-redis-desktop-manager

# Clickhouse
# Linux
export LTS=21.8.4.51
curl -LO https://repo.clickhouse.tech/tgz/lts/clickhouse-common-static-${LTS}.tgz

tar -xzvf clickhouse-common-static-${LTS}.tgz
sudo bash clickhouse-common-static-${LTS}/install/doinst.sh

# mac
#brew tap altinity/clickhouse
#brew install altinity/clickhouse/clickhouse@21.8-altinity-stable

aria2c 'https://builds.clickhouse.com/master/macos/clickhouse'
mv clickhouse ~/bin
chmod a+x ~/bin/clickhouse

# Clickhouse GUI
git clone https://github.com/VKCOM/lighthouse
browser lighthouse/index.html

# textql as an alternative
brew install textql

```

## EXAMPLES

### Genomic features and sliding windows

```shell
# start redis-server
redis-server

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/gars/tests/S288c/

# create gars.env
gars env

# check DB
gars status test

# drop DB
gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv
gars tsv -s 'ctg:*' --range > tests/S288c/ctg.range.tsv

gars-stat tests/S288c/ctg.tsv ctg

# annotate
gars anno tests/S288c/intergenic.yml tests/S288c/ctg.range.tsv -H

# locate an range
gars locate "I(+):1000-1100"

# add features
gars feature tests/S288c/spo11_hot.rg

# sliding windows around ranges
gars fsw

# add ranges
gars range tests/S288c/spo11_hot.rg

# clear
gars clear range

# dump DB to redis-server start dir as dump.rdb
gars status dump

```

### GC-wave

```shell
# start redis-server
redis-server &

# gars
gars env

gars status drop

gars gen tests/S288c/genome.fa.gz --piece 500000

# GC-content of 100 bp sliding window in steps of 1 bp
gars sliding \
    --ctg 'ctg:I:*' \
    --size 100 --step 1 \
    --lag 1000 \
    --threshold 3.0 \
    --influence 1.0 \
    -o tests/S288c/I.gc.tsv

# count of peaks
tsv-summarize tests/S288c/I.gc.tsv \
    -H --group-by signal --count
#signal  count
#0       227242
#-1      2124
#1       753

# merge adjacent windows
tsv-filter tests/S288c/I.gc.tsv -H --ne signal:0 |
    cut -f 1 |
    rgr merge -c 0.8 stdin -o tests/S288c/I.replace.tsv

tsv-filter tests/S288c/I.gc.tsv -H --ne signal:0 |
    rgr replace stdin tests/S288c/I.replace.tsv |
    tsv-uniq -H -f 1 \
    > tests/S288c/I.peaks.tsv

# count of real peaks
tsv-summarize tests/S288c/I.peaks.tsv \
    -H --group-by signal --count
#signal  count
#-1      94
#1       61

gars peak tests/S288c/I.peaks.tsv

# --parallel
gars sliding \
    --ctg 'ctg:I:*' \
    --size 100 --step 10 \
    --lag 100 \
    --threshold 3.0 \
    --influence 1.0 \
    --parallel 1 \
    -o tests/S288c/I.gc.tsv


```

### Env variables

```shell
REDIS_PORT=7379 gars env -o stdout

gars env

gars status test

```

## AUTHOR

Qiang Wang <wang-q@outlook.com>

## LICENSE

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2021.
