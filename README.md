# gams

[![Build](https://github.com/wang-q/gams/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/gams/actions)
[![codecov](https://codecov.io/gh/wang-q/gams/graph/badge.svg?token=LtxYK5Fff0)](https://codecov.io/gh/wang-q/gams)
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
mkdir -p /tmp/cargo
export CARGO_TARGET_DIR=/tmp/cargo
cargo build

# build for CentOS 7
# rustup target add x86_64-unknown-linux-gnu
# pip3 install cargo-zigbuild
cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
ll $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/gams

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

# locate an range
gams locate "I:1000-1050"
gams locate --seq "I:1000-1050"

# add features
gams feature tests/S288c/spo11_hot.rg

gams tsv -s 'feature:*'

# sliding windows around features
cargo run --bin gams fsw

# add rgs
gams rg tests/S288c/SK1.snp.rg

cargo run --bin gams locate --count "I:1000-2000" "II:1000-2000" "Mito:1000-2000"
#I:1000-2000     12
#Mito:1000-2000  0

# annotate
gams anno -H tests/S288c/intergenic.json tests/S288c/ctg.tsv

# clear
gams clear rg

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
and `redis` is expensive. After connecting to redis, a `SET` or `GET` by `gams` consumes about 100
μs. A pipeline of 100 `SET` operations takes almost the same amount of time. If `redis-server`
and `gams` are running on different hosts, the latency is increased for both physical and virtual
NICs. Typical latency for a Gigabit Ethernet is about 200 μs. Thus, for insert operations, `gams`
packages hundreds of operations locally and then passes them to `redis` at once.

As the results of `cargo bench ---bench serialize` show, for a normal
structure, `bincode::serialize()` and `serde_json::to_string()` take 20 ns and 150 ns, respectively,
while `bincode::deserialize()` and `serde_json::from_str()` take about 60 ns and 160 ns,
respectively. These times are insignificant compared to IPC.

Similarly, for genome sequences, `gams` gz-compresses them locally before passing them to `redis`.

### Contents stored in Redis

* `gams` stores key-value pairs in Redis. Keys can be grouped as follows:
    * Basic information about the genome - `top:`
    * Serials - `cnt:`
    * Contigs, a contiguous genomic region - `ctg:`
    * Sequences - `seq:`
    * Bincode, serialized data structure - `bundle:`
    * Indexes - `idx:`

* `gams` uses only one Redis data types, STRING
    * serial - the INCR command parses string values into integers
    * Rust types like Vec<String> are serialized to json using serde
    * Indexes for Ctg, Rg are made by rust_lapper, and serialized to bincode
    * DNA sequences were separated into pieces, gzipped and then stored

* gams naming conventions
    * Rust struct - Ctg, Feature, Rg, Peak
    * Rust variable - serial, chr_id, ctg_id...

| Namespace                 |  Type   | Contents                | Description                                            |
|:--------------------------|:-------:|:------------------------|:-------------------------------------------------------|
| top:common_name           | STRING  |                         | The common name, e.g. S288c, Human                     |
| top:chrs                  |  JSON   | Vec<String>             | Names of each chromosome                               |
| top:chr_len               |  JSON   | BTreeMap<chr_id, usize> | Lengths of each chromosome                             |
|                           |         |                         |                                                        |
| **ctg**                   |         |                         | *A contiguous range on chromosome*                     |
| cnt:ctg:{chr_id}          | INTEGER |                         | Serial number. An internal counter of ctgs on this chr |
| ctg:{chr_id}:{serial}     |  JSON   | Ctg                     | ctg_id => Ctg                                          |
|                           |         | range                   | 1:4000001-4500000                                      |
|                           |         | chr_id                  |                                                        |
|                           |         | chr_start               |                                                        |
|                           |         | chr_end                 |                                                        |
|                           |         | chr_strand              |                                                        |
|                           |         | length                  |                                                        |
| idx:ctg:{chr_id}          | BINARY  | Lapper<u32, String>     | Indexing ctgs to find one                              |
| bundle:ctg:{chr_id}       | BINARY  | BTreeMap<ctg_id, Ctg>   | Retrieves all ctgs of a chr                            |
| seq:{ctg_id}              | BINARY  | Gzipped &[u8]           | Compressed genomic sequence of ctg                     |
|                           |         |                         |                                                        |
| **feature**               |         |                         | *A generic genomic feature of interest*                |
| cnt:feature:{ctg_id}      | INTEGER |                         | Counter of features locating on this ctg               |
| feature:{ctg_id}:{serial} |  JSON   | Feature                 | feature_id => Feature                                  |
|                           |         | range                   |                                                        |
|                           |         | length                  |                                                        |
|                           |         | tag                     |                                                        |
|                           |         |                         |                                                        |
| **rg**                    |         |                         | *For counting overlaps to sliding windows*             |
| cnt:rg:{ctg_id}           | INTEGER |                         | Counter                                                |
| rg:{ctg_id}:{serial}      |  JSON   | Rg                      | range_id => Rg                                         |
|                           |         | range                   |                                                        |
| idx:rg:{ctg_id}           | BINARY  |                         | Indexing rgs to count overlaps                         |
|                           |         |                         |                                                        |
| **peak**                  |         |                         | *Peaks of GC-wave*                                     |
| cnt:peak:{ctg_id}         | INTEGER |                         | Counter                                                |
| peak:{ctg_id}:{serial}    |  JSON   | Peak                    | peak_id => Peak                                        |
|                           |         | length                  |                                                        |
|                           |         | gc                      | GC-content                                             |
|                           |         | signal                  | 1 for crest, -1 for trough                             |
|                           |         | left_wave_length        | Distance to previous peak                              |
|                           |         | left_amplitude          | Difference of GC-content to previous peak              |
|                           |         | left_signal             | Signal of previous peak                                |
|                           |         | right_wave_length       | distance to next peak                                  |
|                           |         | right_amplitude         | Difference of GC-content to next peak                  |
|                           |         | right_signal            | Signal of next peak                                    |

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
