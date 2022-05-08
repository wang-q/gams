# RediSearch

## `redisearch.so`

`redisearch.so` is not easy to build, but we can get it from the docker image.

* Start redis from docker and bring the local directory into it

```shell
docker run -it --rm --entrypoint /bin/sh -v $(pwd):/data redislabs/redisearch

cp /usr/lib/redis/modules/redisearch.so .

```

## ZRANGE and RediSearch

```shell
docker run -p 6379:6379 redislabs/redisearch
#redis-server --loadmodule ~/redisearch.so

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

hyperfine --warmup 1 --export-markdown search.md.tmp \
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

| Command    |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-----------|-----------:|---------:|---------:|------------:|
| ZRANGE     | 18.1 ± 3.2 |     13.3 |     32.9 | 3.65 ± 0.74 |
| redisearch |  5.0 ± 0.5 |      4.2 |     11.8 |        1.00 |
