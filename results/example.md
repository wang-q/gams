# From cli

## `redis-cli`

```shell script
# Inside WSL
# cd /mnt/c/Users/wangq/Scripts/garr

# start redis-server
redis-server --appendonly no --dir tests/S288c/

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/garr/tests/S288c/ --dbfilename dump.rdb

# check config
echo "CONFIG GET *" | redis-cli | grep -e "dir" -e "dbfilename" -A1

# create garr.env
garr env

# check DB
garr status test

# drop DB
redis-cli FLUSHDB

# chr.sizes
faops size tests/S288c/genome.fa.gz > tests/S288c/chr.sizes

# generate DB
#garr gen tests/S288c/chr.sizes
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

faops filter -l 0 tests/S288c/genome.fa.gz stdout |
    paste - - |
    sed 's/^>//' |
    parallel -k -r --col-sep '\t' '
        redis-cli HSET ctg:{1}:1 seq {2}
    '
#redis-cli --raw HSTRLEN ctg:I:1 seq
#redis-cli --raw scan 0 match ctg:I:* type hash

# dump DB to redis-server start dir as dump.rdb
redis-cli SAVE

ps -e | grep redis-server

```

## `gen`

```shell script
# start redis-server
redis-server --appendonly no --dir tests/S288c/

# create garr.env
garr env

# check DB
garr status test

# drop DB
garr status drop

# generate DB
garr gen tests/S288c/genome.fa.gz --piece 100000

garr tsv -s 'ctg:*' > tests/S288c/ctg.tsv

#cargo run stat tests/S288c/ctg.tsv -s templates/ctg-1.sql
#cargo run stat tests/S288c/ctg.tsv -s templates/ctg-2.sql

textql -dlm=tab -header -output-dlm=tab -output-header \
    -sql "$(cat templates/ctg-1.sql)" \
    tests/S288c/ctg.tsv

# add ranges
garr range tests/S288c/spo11_hot.pos.txt

time garr rsw > /dev/null
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
garr pos tests/S288c/spo11_hot.pos.txt tests/S288c/spo11_hot.pos.txt

# dump DB to redis-server start dir as dump.rdb
garr status dump

```

## GC-wave

```shell script
redis-server --appendonly no --dir tests/S288c/

garr status drop

garr gen tests/S288c/genome.fa.gz --piece 500000

garr sliding \
    --ctg 'ctg:I:' \
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

garr wave tests/S288c/I.peaks.tsv

```
