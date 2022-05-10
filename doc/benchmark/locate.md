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
gars locate -f tests/S288c/spo11_hot.pos.txt

# cached index
gars locate --idx "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# ZRANGE
gars locate --zrange "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -r "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate --idx "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate --zrange "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"'

cat locate.md.tmp

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate -f tests/S288c/spo11_hot.pos.txt' \
    'gars locate --idx tests/S288c/spo11_hot.pos.txt' \
    'gars locate --zrange -f tests/S288c/spo11_hot.pos.txt'

cat locate.md.tmp

```

## R7 5800 Windows 11

| Command         | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------------|----------:|---------:|---------:|------------:|
| lapper          | 4.2 ± 0.2 |      3.7 |      6.7 | 1.00 ± 0.07 |
| rebuild; lapper | 4.9 ± 0.3 |      4.3 |      6.2 | 1.16 ± 0.09 |
| idx             | 4.2 ± 0.2 |      3.8 |      5.8 |        1.00 |
| zrange          | 4.3 ± 0.2 |      3.8 |      5.0 | 1.02 ± 0.06 |

| Command | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------|----------:|---------:|---------:|------------:|
| lapper  | 8.2 ± 0.3 |      7.5 |      9.9 | 1.99 ± 0.12 |
| idx     | 4.1 ± 0.2 |      3.7 |      5.2 |        1.00 |
| zrange  | 9.8 ± 0.3 |      8.8 |     11.1 | 2.38 ± 0.14 |

## i7 8700K macOS
