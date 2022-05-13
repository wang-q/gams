# `redis-cli`

A `redis-cli` implementation that is functionally equivalent to `gars gen`

```shell
# start redis-server
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

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
        redis-cli HSET ctg:{1}:1 chr_id {1} chr_start 1 chr_end {2} chr_strand "+" chr_runlist "1-{2}"
        redis-cli ZADD ctg-s:{1} 1 ctg:{1}:1
        redis-cli ZADD ctg-e:{1} {2} ctg:{1}:1
    '

# find a contig contains 1000
# secondary index
redis-cli --raw ZRANGEBYSCORE ctg-s:I 0 1000
redis-cli --raw ZRANGEBYSCORE ctg-e:I 1000 +inf

redis-cli --raw <<EOF
MULTI
ZRANGESTORE tmp:start:I ctg-s:I 0 1000 BYSCORE
ZRANGESTORE tmp:end:I ctg-e:I 1000 +inf BYSCORE
SET comment "ZINTERSTORE tmp:ctg:I 2 tmp:start:I tmp:end:I AGGREGATE MIN"
ZINTER 2 tmp:start:I tmp:end:I AGGREGATE MIN
DEL tmp:start:I tmp:end:I
EXEC
EOF

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
