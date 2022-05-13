# `gars locate`

## ZRANGE and Lapper

```shell
redis-server

gars env

gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

# rebuild the lapper index of ctgs
gars locate -r "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# spo11
gars locate -f tests/S288c/spo11_hot.ranges

# cached index
gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# Deserialize the index on request
gars locate --lapper "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# ZRANGE
gars locate --zrange "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -r "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate --lapper "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate --zrange "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"'

cat locate.md.tmp

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate -f tests/S288c/spo11_hot.ranges' \
    'gars locate --lapper -f tests/S288c/spo11_hot.ranges' \
    'gars locate --zrange -f tests/S288c/spo11_hot.ranges'

cat locate.md.tmp

```

## R7 5800 Windows 11

| Command      | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-------------|----------:|---------:|---------:|------------:|
| idx          | 4.3 ± 0.3 |      3.8 |      6.6 |        1.00 |
| rebuild; idx | 4.5 ± 0.2 |      4.2 |      5.4 | 1.06 ± 0.08 |
| lapper       | 4.3 ± 0.2 |      3.9 |      5.0 | 1.01 ± 0.08 |
| zrange       | 4.4 ± 0.2 |      3.9 |      5.1 | 1.02 ± 0.08 |

| Command | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------|----------:|---------:|---------:|------------:|
| idx     | 5.7 ± 0.4 |      4.6 |      8.7 |        1.00 |
| lapper  | 8.3 ± 0.3 |      7.5 |     10.3 | 1.47 ± 0.12 |
| zrange  | 9.9 ± 0.4 |      8.9 |     12.0 | 1.75 ± 0.15 |

## i7 8700K macOS
