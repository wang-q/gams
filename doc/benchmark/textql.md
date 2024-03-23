# `gars-stat`, `gars-sql` and `textql`

## Install

```shell
# gars-stat, gars-sql
cargo install --force --path . --features stat

# textql
brew install textql

```

## ctg

```shell
redis-server

gars env

gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv

gars-stat tests/S288c/ctg.tsv ctg

gars-sql tests/S288c/ctg.tsv templates/ctg-2.sql

textql -dlm=tab -header -output-dlm=tab -output-header \
    -sql "$(cat templates/ctg-2.sql)" \
    tests/S288c/ctg.tsv

hyperfine --warmup 1 --export-markdown stat.md.tmp \
    -n gars-stat \
    'gars-stat tests/S288c/ctg.tsv ctg > /dev/null' \
    -n gars-sql \
    'gars-sql tests/S288c/ctg.tsv templates/ctg-2.sql > /dev/null' \
    -n textql \
    '
    textql -dlm=tab -header -output-dlm=tab -output-header \
        -sql "$(cat templates/ctg-2.sql)" \
        tests/S288c/ctg.tsv > /dev/null
    '

cat stat.md.tmp

```

### R7 5800 Windows 11

| Command   | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------|----------:|---------:|---------:|------------:|
| gars-stat | 4.3 ± 0.2 |      3.9 |      5.3 |        1.00 |
| textql    | 5.8 ± 0.2 |      5.0 |      6.6 | 1.35 ± 0.07 |

### i5-12500H Windows 11 WSL

| Command     |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------|-----------:|---------:|---------:|------------:|
| `gars-stat` |  8.7 ± 0.8 |      7.3 |     15.8 | 1.35 ± 0.92 |
| `gars-sql`  | 15.0 ± 2.6 |     12.8 |     31.4 | 2.32 ± 1.62 |
| `textql`    |  6.5 ± 4.4 |      4.8 |     94.4 |        1.00 |

### Apple M2 macOS 13.4

| Command     | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------|----------:|---------:|---------:|------------:|
| `gars-stat` | 2.6 ± 0.5 |      2.3 |      9.2 |        1.00 |
| `textql`    | 4.6 ± 0.9 |      4.2 |     14.5 | 1.75 ± 0.50 |
