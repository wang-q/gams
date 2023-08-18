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

### R7 5800 Windows 11

| Command        |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------|-----------:|---------:|---------:|------------:|
| `idx`          | 11.4 ± 0.9 |     10.1 |     14.1 |        1.00 |
| `rebuild; idx` | 11.7 ± 0.9 |     10.6 |     14.8 | 1.03 ± 0.11 |
| `lapper`       | 15.4 ± 1.1 |     13.4 |     18.5 | 1.36 ± 0.14 |
| `zrange`       | 17.3 ± 1.3 |     15.1 |     20.6 | 1.53 ± 0.16 |

### E5-2680 v3 RHEL 7.7

| Command        |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------|-----------:|---------:|---------:|------------:|
| `idx`          |  6.9 ± 0.3 |      6.5 |      9.0 |        1.00 |
| `rebuild; idx` |  7.1 ± 0.2 |      6.7 |      9.0 | 1.03 ± 0.06 |
| `lapper`       |  8.4 ± 0.3 |      8.0 |      9.8 | 1.22 ± 0.07 |
| `zrange`       | 10.2 ± 0.4 |      9.7 |     11.7 | 1.48 ± 0.08 |

### Apple M2 macOS 13.4

| Command        | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------|----------:|---------:|---------:|------------:|
| `idx`          | 3.9 ± 0.8 |      3.3 |     12.8 |        1.00 |
| `rebuild; idx` | 4.5 ± 0.2 |      3.7 |      5.2 | 1.14 ± 0.23 |
| `lapper`       | 7.1 ± 0.8 |      5.2 |     10.2 | 1.82 ± 0.41 |
| `zrange`       | 7.2 ± 0.6 |      5.9 |      8.7 | 1.83 ± 0.39 |


## RediSearch

### `redisearch.so`

`redisearch.so` is not easy to build, but we can get it from the docker image.

* Start redis from docker and bring the local directory into it

```shell
docker run -it --rm --entrypoint /bin/sh -v $(pwd):/data redislabs/redisearch

cp /usr/lib/redis/modules/redisearch.so .

```

### ZRANGE and RediSearch

```shell
docker run -p 6379:6379 redislabs/redisearch
# redis-server --loadmodule ~/redisearch.so

gars env

gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

# ZRANGE
redis-cli --raw <<EOF
MULTI
ZRANGESTORE tmp:start:I ctg-s:I 0 1000 BYSCORE
ZRANGESTORE tmp:end:I ctg-e:I 1000 +inf BYSCORE
SET comment "ZINTERSTORE tmp:ctg:I 2 tmp:start:I tmp:end:I AGGREGATE MIN"
ZINTER 2 tmp:start:I tmp:end:I AGGREGATE MIN
DEL tmp:start:I tmp:end:I
EXEC
EOF

# RediSearch
redis-cli --raw FT.CREATE idx:ctg:I ON HASH PREFIX 1 ctg:I \
    SCHEMA chr_start NUMERIC \
    chr_end NUMERIC

redis-cli --raw FT.SEARCH idx:ctg:I "@chr_start:[-inf 1000] @chr_end:[1000 inf]" RETURN 0

hyperfine --export-markdown search.md.tmp \
    '
redis-cli --raw <<EOF
MULTI
ZRANGESTORE tmp:start:I ctg-start:I 0 1000 BYSCORE
ZRANGESTORE tmp:end:I ctg-end:I 1000 +inf BYSCORE
SET comment "ZINTERSTORE tmp:ctg:I 2 tmp:start:I tmp:end:I AGGREGATE MIN"
ZINTER 2 tmp:start:I tmp:end:I AGGREGATE MIN
DEL tmp:start:I tmp:end:I
EXEC
EOF
    ' \
    'redis-cli --raw FT.SEARCH idx:ctg:I "@chr_start:[-inf 1000] @chr_end:[1000 inf]" RETURN 0'


```

### R7 5800 Windows 11

* docker

| Command    | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-----------|----------:|---------:|---------:|------------:|
| ZRANGE     | 5.0 ± 0.3 |      4.3 |      7.7 | 4.81 ± 0.52 |
| redisearch | 1.0 ± 0.1 |      0.8 |      2.2 |        1.00 |

* wsl

| Command    | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-----------|----------:|---------:|---------:|------------:|
| ZRANGE     | 3.2 ± 0.2 |      2.9 |      4.5 | 3.95 ± 0.48 |
| redisearch | 0.8 ± 0.1 |      0.6 |      2.1 |        1.00 |

### i7 8700K macOS

* docker

| Command    |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-----------|-----------:|---------:|---------:|------------:|
| ZRANGE     | 18.1 ± 3.2 |     13.3 |     32.9 | 3.65 ± 0.74 |
| redisearch |  5.0 ± 0.5 |      4.2 |     11.8 |        1.00 |
