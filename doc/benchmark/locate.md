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

# ZRANGE
gars locate -z "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -r "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"' \
    'gars locate -z "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"'

cat locate.md.tmp

```

## R7 5800 Windows 11

| Command             | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------------------|----------:|---------:|---------:|------------:|
| find_one_l          | 4.3 ± 0.2 |      3.8 |      6.2 |        1.00 |
| rebuild; find_one_l | 4.8 ± 0.2 |      4.3 |      5.6 | 1.11 ± 0.07 |
| find_one_z          | 4.3 ± 0.2 |      3.9 |      5.7 | 1.01 ± 0.07 |

## i7 8700K macOS
