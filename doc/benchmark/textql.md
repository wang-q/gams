# `gars-stat` and `textql`

## Install

```shell
# gars-stat
cargo install --force --path . --features build-stat

# textql
brew install textql

```

## ctg

```shell
docker run -p 6379:6379 redislabs/redisearch
#redis-server --loadmodule ~/redisearch.so

gars env

gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv

gars-stat tests/S288c/ctg.tsv ctg

textql -dlm=tab -header -output-dlm=tab -output-header \
    -sql "$(cat templates/ctg-2.sql)" \
    tests/S288c/ctg.tsv

hyperfine --warmup 1 --export-markdown stat.md.tmp \
    'gars-stat tests/S288c/ctg.tsv ctg > /dev/null' \
    '
    textql -dlm=tab -header -output-dlm=tab -output-header \
        -sql "$(cat templates/ctg-2.sql)" \
        tests/S288c/ctg.tsv > /dev/null
    '


```

| Command   | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------|----------:|---------:|---------:|------------:|
| gars-stat | 5.4 ± 0.3 |      4.8 |      6.5 |        1.00 |
| textql    | 8.4 ± 0.3 |      7.9 |     10.0 | 1.57 ± 0.10 |
