# From cli

## `gen`

```shell script
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
#redis-cli FLUSHDB
garr status drop

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
#redis-cli SAVE
garr status dump

ps -e | grep redis-server

```

## `pos`

```shell script
cat tests/S288c/spo11_hot.pos.txt |
    parallel -k -r --col-sep ':|\-' '
        echo {1} {2} {3}

    '

```
