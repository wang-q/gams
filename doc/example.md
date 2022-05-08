# `redis-cli`

## `gen` and `rsw`

```shell script
# start redis-server
rm tests/S288c/dump.rdb
redis-server --appendonly no --dir tests/S288c/

# start with dump file
# redis-server --appendonly no --dir ~/Scripts/rust/gars/tests/S288c/ --dbfilename dump.rdb

# create gars.env
gars env

# check DB
gars status test

# drop DB
gars status drop

# generate DB
gars gen tests/S288c/genome.fa.gz --piece 100000

gars tsv -s 'ctg:*' > tests/S288c/ctg.tsv

gars-stat tests/S288c/ctg.tsv ctg

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
