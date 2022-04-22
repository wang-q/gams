# `redis-cli`

## ZRANGE and redisearch

```shell
# start redis-server
rm tests/S288c/dump.rdb
redis-server --loadmodule ~/redisearch.so --appendonly no --dir tests/S288c/

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/gars/tests/S288c/ --dbfilename dump.rdb

# check config
echo "CONFIG GET *" | redis-cli | grep -e "dir" -e "dbfilename" -A1

# drop existing DB
redis-cli FLUSHDB

# chr.sizes
faops size tests/S288c/genome.fa.gz > tests/S288c/chr.sizes

# generate DB
redis-cli SET common_name S288c

cat tests/S288c/chr.sizes |
    parallel -k -r --col-sep '\t' '
        redis-cli HSET chr {1} {2}
    '
redis-cli --raw HLEN chr |
    xargs -I{} echo There are {} chromosomes

cat tests/S288c/chr.sizes |
    parallel -k -r --col-sep '\t' '
        redis-cli HSET ctg:{1}:1 chr_name {1} chr_start 1 chr_end {2} chr_strand "+" chr_runlist "1-{2}"
        redis-cli ZADD ctg-start:{1} 1 ctg:{1}:1
        redis-cli ZADD ctg-end:{1} {2} ctg:{1}:1
    '

# find a contig contains 1000
# secondary index
redis-cli --raw ZRANGEBYSCORE ctg-start:I 0 1000
redis-cli --raw ZRANGEBYSCORE ctg-end:I 1000 +inf

redis-cli --raw <<EOF
MULTI
ZRANGESTORE tmp:start:I ctg-start:I 0 1000 BYSCORE
ZRANGESTORE tmp:end:I ctg-end:I 1000 +inf BYSCORE
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

for CHR in $(cat tests/S288c/chr.sizes | cut -f 1); do
    echo ${CHR}
    echo HSET "ctg:${CHR}:1" seq $(
            faops one -l 0 tests/S288c/genome.fa.gz ${CHR} stdout | sed "1d"
        ) |
        redis-cli
done

#redis-cli --raw HSTRLEN ctg:I:1 seq
#redis-cli --raw scan 0 match ctg:I:* type hash

# dump DB to redis-server start dir as dump.rdb
redis-cli SAVE

ps -e | grep redis-server

```

| Command    | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-----------|----------:|---------:|---------:|------------:|
| ZRANGE     | 3.3 ± 0.7 |      2.3 |     19.3 | 3.53 ± 1.13 |
| redisearch | 0.9 ± 0.2 |      0.6 |      3.3 |        1.00 |

## `gen` and `rsw`

```shell script
# start redis-server
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

# create gars.env
gars env

# check DB
gars status test

# drop DB
gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv

#cargo run stat tests/S288c/ctg.tsv -s templates/ctg-1.sql
#cargo run stat tests/S288c/ctg.tsv -s templates/ctg-2.sql

textql -dlm=tab -header -output-dlm=tab -output-header \
    -sql "$(cat templates/ctg-2.sql)" \
    tests/S288c/ctg.tsv

gars stat tests/S288c/ctg.tsv ctg

hyperfine --warmup 1 --export-markdown stat.md.tmp \
    '
    textql -dlm=tab -header -output-dlm=tab -output-header \
        -sql "$(cat templates/ctg-2.sql)" \
        tests/S288c/ctg.tsv
    ' \
    'gars stat tests/S288c/ctg.tsv ctg'

# add ranges
gars range tests/S288c/spo11_hot.pos.txt

time gars rsw > /dev/null
# w/o gc_stat()
# get_gc_content()
#real    0m1.444s
#user    0m0.174s
#sys     0m0.838s

# ctg_gc_content()
#real    0m0.774s
#user    0m0.406s
#sys     0m0.289s

# w/ gc_stat()
# get_gc_content()
#real    0m3.476s
#user    0m0.436s
#sys     0m2.037s

# ctg_gc_content()
#real    0m2.349s
#user    0m1.041s
#sys     0m1.102s

# add pos
gars pos tests/S288c/spo11_hot.pos.txt tests/S288c/spo11_hot.pos.txt

# dump DB to redis-server start dir as dump.rdb
gars status dump

```

| Command   | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------|----------:|---------:|---------:|------------:|
| textql    | 5.1 ± 0.4 |      4.2 |      6.8 | 1.05 ± 0.18 |
| gars stat | 4.9 ± 0.8 |      4.2 |     19.6 |        1.00 |

## GC-wave

```shell script
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

gars status drop

gars gen tests/S288c/genome.fa.gz --piece 500000

gars sliding \
    --ctg 'ctg:I:*' \
    --size 100 --step 1 \
    --lag 1000 \
    --threshold 3.0 \
    --influence 1.0 \
    -o tests/S288c/I.gc.tsv

Rscript templates/peak.tera.R \
    --lag 1000 \
    --threshold 3.0 \
    --influence 1.0 \
    --infile tests/S288c/I.gc.tsv \
    --outfile tests/S288c/I.R.tsv

tsv-summarize tests/S288c/I.gc.tsv \
    -H --group-by signal --count
#signal  count
#0       227242
#-1      2124
#1       753

tsv-summarize tests/S288c/I.R.tsv \
    -H --group-by signal --count
#signal  count
#0       227317
#-1      2079
#1       723

tsv-filter tests/S288c/I.gc.tsv -H --ne signal:0 |
    cut -f 1 |
    linkr merge -c 0.8 stdin -o tests/S288c/I.replace.tsv

tsv-filter tests/S288c/I.gc.tsv -H --ne signal:0 |
    ovlpr replace stdin tests/S288c/I.replace.tsv |
    tsv-uniq -H -f 1 \
    > tests/S288c/I.peaks.tsv

tsv-summarize tests/S288c/I.peaks.tsv \
    -H --group-by signal --count
#signal  count
#-1      94
#1       61

gars wave tests/S288c/I.peaks.tsv

```
