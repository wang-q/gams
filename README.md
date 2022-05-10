# `gars`

![Publish](https://github.com/wang-q/gars/workflows/Publish/badge.svg)
![Build](https://github.com/wang-q/gars/workflows/Build/badge.svg)

`gars` - **G**enome **A**nalyst with **R**ust and redi**S**

## INSTALL

Current release: 0.2.1

```shell
cargo install --force --offline --path .

# test
cargo test -- --test-threads=1

# gars-stat
cargo install --force --path . --features build-stat

```

## SYNOPSIS

```text
$ gars help
gars 0.2.0
wang-q <wang-q@outlook.com>
Genome Analyst with Rust and rediS

USAGE:
    gars [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    env        Create a .env file
    gen        Generate the database from (gzipped) fasta files
    help       Print this message or the help of the given subcommand(s)
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

* R (4.2) packages

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
# brew install --cask another-redis-desktop-manager

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

### `gen` and `rsw`

```shell
# start redis-server
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/gars/tests/S288c/ --dbfilename dump.rdb

# create gars.env
gars env

# check DB
gars status test

# drop DB
gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv

gars-stat tests/S288c/ctg.tsv ctg

# locate an range
gars locate "I(+):1000-1100"

# add ranges
gars range tests/S288c/spo11_hot.pos.txt

# sliding windows around ranges
gars rsw

# dump DB to redis-server start dir as dump.rdb
gars status dump

```

### GC-wave

```shell
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

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
    linkr merge -c 0.8 stdin -o tests/S288c/I.replace.tsv

tsv-filter tests/S288c/I.gc.tsv -H --ne signal:0 |
    ovlpr replace stdin tests/S288c/I.replace.tsv |
    tsv-uniq -H -f 1 \
    > tests/S288c/I.peaks.tsv

# count of real peaks
tsv-summarize tests/S288c/I.peaks.tsv \
    -H --group-by signal --count
#signal  count
#-1      94
#1       61

gars wave tests/S288c/I.peaks.tsv

```

### Env variables

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
