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

## Atha

* genome

```shell script
mkdir -p ~/data/garr/Atha/
cd ~/data/garr/Atha/

# download
aria2c -j 4 -x 4 -s 2 --file-allocation=none -c \
    http://ftp.ensemblgenomes.org/pub/release-45/plants/fasta/arabidopsis_thaliana/dna/Arabidopsis_thaliana.TAIR10.dna_sm.toplevel.fa.gz

# chromosomes
gzip -d -c *dna_sm.toplevel* > toplevel.fa

faops count toplevel.fa |
    perl -nla -e '
        next if $F[0] eq 'total';
        print $F[0] if $F[1] > 50000;
        print $F[0] if $F[1] > 5000  and $F[6]/$F[1] < 0.05;
    ' |
    uniq > listFile
faops some toplevel.fa listFile stdout |
    faops filter -N stdin stdout |
    faops split-name stdin .
rm toplevel.fa listFile

mv Mt.fa Mt.fa.skip
mv Pt.fa Pt.fa.skip

# .fa.gz
cat {1..5}.fa |
    gzip -9 \
    > genome.fa.gz
faops size genome.fa.gz > chr.sizes

```

* T-DNA

```shell script
mkdir -p ~/data/garr/TDNA/
cd ~/data/garr/TDNA/

for name in CSHL FLAG MX RATM; do
    aria2c -j 4 -x 4 -s 2 --file-allocation=none -c \
        http://natural.salk.edu/database/tdnaexpress/T-DNA.${name}
done

# Convert to positions
for name in CSHL FLAG MX RATM; do
    cat T-DNA.${name} |
         perl -nla -e '
            @F >= 2 or next;
            next unless $F[1];

            my ( $chr, $pos ) = split /:/, $F[1];
            $chr =~ s/chr0?//i;
            $pos =~ s/^0+//;
            next unless $chr =~ /^\d+$/;

            print "$chr:$pos";
        ' \
        > T-DNA.${name}.pos.txt;
done

```

* `garr`

```shell script
# start redis-server
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env

garr status drop

garr gen genome.fa.gz --piece 500000

for name in CSHL FLAG MX RATM; do
    garr pos ../TDNA/T-DNA.${name}.pos.txt
done

garr status dump

garr tsv -s 'ctg:*'
garr tsv -s 'ctg:*' -f length

```

* Ranges and rsw

```shell script
rm ~/data/garr/Atha/dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/
#keydb-server --appendonly no --dir ~/data/garr/Atha/
# keydb is as fast/slow as redis

cd ~/data/garr/Atha/

garr env

garr status drop

time garr gen genome.fa.gz --piece 500000
#real    0m1.520s
#user    0m0.582s
#sys     0m0.407s

garr tsv -s 'ctg:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n |
    cut -f 1 \
    > ctg.lst

time parallel -j 4 -k --line-buffer '
    echo {}
    garr range ../TDNA/T-DNA.{}.pos.txt --tag {}
    ' ::: CSHL # FLAG MX RATM
# redis
# RATM
#real    0m14.055s
#user    0m1.819s
#sys     0m6.357s
# 4 files
#real    0m40.654s
#user    0m11.503s
#sys     0m21.387s

# keydb
# RATM
#real    0m14.228s
#user    0m1.792s
#sys     0m6.314s
# 4 files
#real    0m42.186s
#user    0m11.481s
#sys     0m21.391s

time cat ctg.lst |
    parallel -j 2 -k --line-buffer '
        garr rsw --ctg {}
        ' |
    tsv-uniq \
    > T-DNA.rsw.tsv
# CSHL
# -j 4
#real    7m43.384s
#user    24m58.916s
#sys     3m42.415s
# -j 2
#real    13m38.805s
#user    21m56.417s
#sys     4m34.154s

```

* GC-wave

```shell script
cd ~/data/garr/Atha/

garr env

garr status drop

garr gen genome.fa.gz --piece 500000

time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        garr sliding \
            --ctg {} \
            --size 100 --step 1 \
            --lag 1000 \
            --threshold 3.0 \
            --influence 1.0 \
            -o stdout |
            tsv-filter -H --ne signal:0 \
            > {.}.gc.tsv

        cat {.}.gc.tsv |
            cut -f 1 |
            linkr merge -c 0.8 stdin -o {.}.replace.tsv

        cat {.}.gc.tsv |
            ovlpr replace stdin {.}.replace.tsv |
            tsv-uniq -H -f 1 \
            > {.}.peaks.tsv

        tsv-summarize {.}.peaks.tsv \
            -H --group-by signal --count

        rm {.}.gc.tsv {.}.replace.tsv
    '
#real    5m58.731s
#user    23m47.703s
#sys     0m16.114s

tsv-append ctg:*.peaks.tsv -H > Atha.peaks.tsv
rm ctg:*.peaks.tsv

tsv-summarize Atha.peaks.tsv \
    -H --group-by signal --count
#signal  count
#1       32211
#-1      26821

time garr wave Atha.peaks.tsv
#real    4m27.902s
#user    0m26.255s
#sys     2m31.036s

garr tsv -s "peak:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > Atha.wave.tsv

cat Atha.wave.tsv |
    tsv-summarize -H --count
# 59032

tsv-filter Atha.wave.tsv -H --or \
    --le left_wave_length:0 --le right_wave_length:0 |
    tsv-summarize -H --count
# 12927

```

* benchmarks

```shell script
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env

hyperfine --warmup 1 --export-markdown garr.md.tmp \
    '
        garr status drop;
        garr gen genome.fa.gz --piece 500000;
    ' \
    '
        garr status drop;
        garr gen genome.fa.gz --piece 500000;
        garr pos ../TDNA/T-DNA.CSHL.pos.txt;
    ' \
    '
        garr status drop;
        garr gen genome.fa.gz --piece 500000;
        garr range ../TDNA/T-DNA.CSHL.pos.txt --tag CSHL;
    ' \
    '
        garr status drop;
        garr gen genome.fa.gz --piece 500000;
        garr sliding --size 100 --step 20 --lag 50 |
            tsv-filter -H --ne signal:0 > /dev/null;
    '

cat garr.md.tmp

```

| Command               |       Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:----------------------|----------------:|---------:|---------:|-------------:|
| `drop; gen;`          |    813.8 ± 18.5 |    783.4 |    834.4 |         1.00 |
| `drop; gen; pos;`     |  8203.5 ± 166.7 |   8051.7 |   8475.7 | 10.08 ± 0.31 |
| `drop; gen; range;`   | 20045.7 ± 474.6 |  19218.6 |  21041.7 | 24.63 ± 0.81 |
| `drop; gen; sliding;` |   7580.7 ± 72.6 |   7467.1 |   7705.8 |  9.31 ± 0.23 |

