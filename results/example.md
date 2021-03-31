# From cli

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

# dump DB to redis-server start dir as dump.rdb
redis-cli SAVE

ps -e | grep redis-server

```

