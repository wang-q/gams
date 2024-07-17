# gams

[![Build](https://github.com/wang-q/gams/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/gams/actions)
[![codecov](https://codecov.io/gh/wang-q/gams/branch/main/graph/badge.svg?token=LtxYK5Fff0)](https://codecov.io/gh/wang-q/gams)
[![license](https://img.shields.io/github/license/wang-q/intspan)](https://github.com//wang-q/intspan)

`gams` - **G**enome **A**nalyst with in-**M**emory **S**torage

## Install

Current release: 0.3.1

```shell
cargo install --path . --force --offline

# test
cargo test -- --test-threads=1

# gams-stat
cargo install --force --path . --features stat --offline

# build under WSL 2
export CARGO_TARGET_DIR=/tmp
cargo build

```

## Synopsis

### `gams help`

```text
Genome Analyst with in-Memory Storage

Usage: gams [COMMAND]

Commands:
  env      Create a .env file
  status   Test Redis config and connection
  gen      Generate the database from (gzipped) fasta files
  locate   Locate the given ranges to the corresponding ctgs
  range    Add range files for counting
  clear    Clear some parts from Redis
  feature  Add genomic features from a range file
  fsw      Sliding windows around features
  anno     Annotate anything that contains a ctg_id and a range
  sliding  Sliding windows along a chromosome
  peak     Add peaks of GC-waves
  tsv      Export Redis hashes to a tsv file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

### `gams-stat --help`

```text
gams-stat 0.3.1
wang-q <wang-q@outlook.com>
Build-in stats for gams

USAGE:
    gams-stat [OPTIONS] <infile> <query>

ARGS:
    <infile>    Sets the input file to use
    <query>     Query name [default: ctg]

OPTIONS:
    -h, --help                 Print help information
    -o, --outfile <outfile>    Output filename. [stdout] for screen [default: stdout]
    -V, --version              Print version information

```

## Examples

### Genomic features and sliding windows

```shell
# start redis-server
redis-server

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/gams/tests/S288c/

# create gams.env
gams env

# check DB
gams status test

# drop DB
gams status drop

# generate DB
gams gen tests/S288c/genome.fa.gz --piece 100000

gams tsv -s 'ctg:*' > tests/S288c/ctg.tsv
gams tsv -s 'ctg:I:*'

gams-stat tests/S288c/ctg.tsv ctg

# annotate
gams anno -H tests/S288c/intergenic.json tests/S288c/ctg.tsv

# locate an range
cargo run --bin gams locate "I:1000-1050"
cargo run --bin gams locate --seq "I:1000-1050"

# add features
gams feature tests/S288c/spo11_hot.rg

# sliding windows around ranges
gams fsw

# add ranges
gams range tests/S288c/spo11_hot.rg tests/S288c/spo11_hot.rg

# clear
gams clear range

# dump DB to redis-server start dir as dump.rdb
gams status dump

```

### GC-wave

```shell
# start redis-server
redis-server &

# gams
gams env

gams status drop

gams gen tests/S288c/genome.fa.gz --piece 500000

# GC-content of 100 bp sliding window in steps of 1 bp
gams sliding \
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

gams peak tests/S288c/I.peaks.tsv

# --parallel
gams sliding \
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
# change redis port
REDIS_PORT=7379 gams env -o stdout

gams env

gams status test

```

## Designing concepts

`Redis` has a low operating cost, but the inter-process communication (IPC) between `gams`
and `redis` is expensive. After connecting to redis, a `SET` or `HSET` by `gams` consumes about 100
μs. A pipeline of 100 `HSET` operations takes almost the same amount of time. If `redis-server`
and `gams` are running on different hosts, the latency is increased for both physical and virtual
NICs. Typical latency for a Gigabit Ethernet is about 200 μs. Thus, for insert operations, `gams`
packages hundreds of operations locally and then passes them to `redis` at once.

For complex data structures, a local `bincode::serialize()` takes about 50 ns,
and `bincode::deserialize()` about 100 ns, which is insignificant compared to IPC. Similarly, for
genome sequences, `gams` gz-compresses them locally before passing them to `redis`.

### Contents stored in Redis

| Namespace                 |  Type   | Fields            | Description                                                                  |
|:--------------------------|:-------:|:------------------|:-----------------------------------------------------------------------------|
| common_name               | STRING  |                   | The common name, e.g. S288c, Human                                           |
| chr                       |  HASH   |                   | Length of each chromosome                                                    |
|                           |         | chr_id            |                                                                              |
| **ctg**                   |         |                   | *A contiguous range on chromosome*                                           |
| cnt:ctg:{chr_id}          | INTEGER |                   | Serial number. An internal counter of ctgs on this chr                       |
| ctg:{chr_id}:{serial}     |  HASH   |                   | ID, ctg_id                                                                   |
|                           |         | range             | 1:4000001-4500000                                                            |
|                           |         | chr_id            |                                                                              |
|                           |         | chr_start         |                                                                              |
|                           |         | chr_end           |                                                                              |
|                           |         | chr_strand        |                                                                              |
|                           |         | length            | All other namespaces of type HASH contain these fields                       |
| idx:ctg:{chr_id}          | BINARY  |                   | A serialized structure of `Lapper<u32, String>` for indexing ctgs on chr     |
| bin:ctg:{chr_id}          | BINARY  |                   | A serialized structure of `BTreeMap<String, Ctg>`                            |
| seq:{ctg_id}              | BINARY  |                   | Compressed genomic sequence of ctg                                           |
| **feature**               |         |                   | *A generic genomic feature of interest*                                      |
| cnt:feature:{ctg_id}      | INTEGER |                   | Serial number. Counter of features locating on this ctg                      |
| feature:{ctg_id}:{serial} |  HASH   |                   | ID, feature_id                                                               |
|                           |         | ...               | Standard fields similar to ctg                                               |
|                           |         | tag               | Feature tags                                                                 |
| bin:feature:{ctg_id}      |   SET   |                   | A Redis SET contains all serialized features on this ctg                     |
| **range**                 |         |                   | *Genomic features used for counting, i.e., relationship to certain features* |
| cnt:range:{ctg_id}        | INTEGER |                   | Serial number. Counter of ranges locating on this ctg                        |
| range:{ctg_id}:{serial}   |  HASH   |                   | ID, range_id                                                                 |
|                           |         | ...               | Standard fields similar to ctg                                               |
| idx:range:{ctg_id}        | BINARY  |                   | A serialized structure of `Lapper<u32, String>` for indexing ranges on ctg   |
| **peak**                  |         |                   | Peaks of GC-wave                                                             |
| cnt:peak:{ctg_id}         | INTEGER |                   | Serial number. Counter of peaks locating on this ctg                         |
| peak:{ctg_id}:{serial}    |  HASH   |                   | ID, peak_id                                                                  |
|                           |         | ...               | Standard fields similar to ctg                                               |
|                           |         | signal            | 1 for crest, -1 for trough                                                   |
|                           |         | gc                | GC-content                                                                   |
|                           |         | left_signal       | Signal of previous peak                                                      |
|                           |         | left_wave_length  | Distance to previous peak                                                    |
|                           |         | left_amplitude    | Difference of GC-content to previous peak                                    |
|                           |         | right_signal      | Signal of next peak                                                          |
|                           |         | right_wave_length | distance to next peak                                                        |
|                           |         | right_amplitude   | Difference of GC-content to next peak                                        |

## Runtime dependencies

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

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2021.
