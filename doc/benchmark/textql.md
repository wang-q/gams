# `gams-stat`, `gams-sql` and `textql`

## Install

```shell
# gams-stat, gams-sql
cargo install --force --path . --features stat

# textql
brew install textql

```

## ctg

```shell
redis-server

gams env

gams status drop

# generate DB
gams gen tests/S288c/genome.fa.gz --piece 100000

gams tsv -s 'ctg:*' > tests/S288c/ctg.tsv

gams-stat tests/S288c/ctg.tsv ctg

gams-sql tests/S288c/ctg.tsv templates/ctg-2.sql

textql -dlm=tab -header -output-dlm=tab -output-header \
    -sql "$(cat templates/ctg-2.sql)" \
    tests/S288c/ctg.tsv

hyperfine --warmup 1 --export-markdown stat.md.tmp \
    -n gams-stat \
    'gams-stat tests/S288c/ctg.tsv ctg > /dev/null' \
    -n gams-sql \
    'gams-sql tests/S288c/ctg.tsv templates/ctg-2.sql > /dev/null' \
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
| gams-stat | 4.3 ± 0.2 |      3.9 |      5.3 |        1.00 |
| textql    | 5.8 ± 0.2 |      5.0 |      6.6 | 1.35 ± 0.07 |

### i5-12500H Windows 11 WSL

| Command     |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------|-----------:|---------:|---------:|------------:|
| `gams-stat` |  8.7 ± 0.8 |      7.3 |     15.8 | 1.35 ± 0.92 |
| `gams-sql`  | 15.0 ± 2.6 |     12.8 |     31.4 | 2.32 ± 1.62 |
| `textql`    |  6.5 ± 4.4 |      4.8 |     94.4 |        1.00 |

### Apple M2 macOS 13.4

| Command     | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------|----------:|---------:|---------:|------------:|
| `gams-stat` | 2.6 ± 0.5 |      2.3 |      9.2 |        1.00 |
| `textql`    | 4.6 ± 0.9 |      4.2 |     14.5 | 1.75 ± 0.50 |
