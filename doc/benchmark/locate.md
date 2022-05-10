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

# ZRANGE
gars locate -z "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -r "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -z "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"'

cat locate.md.tmp

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate -f tests/S288c/spo11_hot.pos.txt' \
    'gars locate -z -f tests/S288c/spo11_hot.pos.txt'

cat locate.md.tmp

```

## R7 5800 Windows 11

| Command         | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------------|----------:|---------:|---------:|------------:|
| lapper          | 4.3 ± 0.2 |      3.8 |      6.2 |        1.00 |
| rebuild; lapper | 4.8 ± 0.2 |      4.3 |      5.6 | 1.11 ± 0.07 |
| zrange          | 4.3 ± 0.2 |      3.9 |      5.7 | 1.01 ± 0.07 |

| Command | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------|----------:|---------:|---------:|------------:|
| lapper  | 8.3 ± 0.3 |      7.4 |     10.0 |        1.00 |
| zrange  | 9.8 ± 0.4 |      9.0 |     13.8 | 1.17 ± 0.06 |

## i7 8700K macOS
