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

# cached index
gars locate "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# Deserialize the index on request
gars locate --lapper "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# ZRANGE
gars locate --zrange "I(+):1000-1100" "II:1000-1100" "Mito:1000-1100"

# spo11
gars locate -f tests/S288c/spo11_hot.rg

hyperfine -N --export-markdown locate.md.tmp \
    -n idx \
    'gars locate -f tests/S288c/spo11_hot.rg' \
    -n "rebuild; idx" \
    'gars locate -r -f tests/S288c/spo11_hot.rg' \
    -n lapper \
    'gars locate --lapper -f tests/S288c/spo11_hot.rg' \
    -n zrange \
    'gars locate --zrange -f tests/S288c/spo11_hot.rg'

cat locate.md.tmp

```

## R7 5800 Windows 11

| Command        |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------|-----------:|---------:|---------:|------------:|
| `idx`          | 11.4 ± 0.9 |     10.1 |     14.1 |        1.00 |
| `rebuild; idx` | 11.7 ± 0.9 |     10.6 |     14.8 | 1.03 ± 0.11 |
| `lapper`       | 15.4 ± 1.1 |     13.4 |     18.5 | 1.36 ± 0.14 |
| `zrange`       | 17.3 ± 1.3 |     15.1 |     20.6 | 1.53 ± 0.16 |

## E5-2680 v3 RHEL 7.7

| Command        |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------|-----------:|---------:|---------:|------------:|
| `idx`          |  6.9 ± 0.3 |      6.5 |      9.0 |        1.00 |
| `rebuild; idx` |  7.1 ± 0.2 |      6.7 |      9.0 | 1.03 ± 0.06 |
| `lapper`       |  8.4 ± 0.3 |      8.0 |      9.8 | 1.22 ± 0.07 |
| `zrange`       | 10.2 ± 0.4 |      9.7 |     11.7 | 1.48 ± 0.08 |
