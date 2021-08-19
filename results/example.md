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

# add ranges
garr range tests/S288c/spo11_hot.pos.txt

# add pos
garr pos tests/S288c/spo11_hot.pos.txt tests/S288c/spo11_hot.pos.txt

# dump DB to redis-server start dir as dump.rdb
garr status dump

```

## Sliding


```shell script
redis-server --appendonly no --dir tests/S288c/

garr sliding --size 100 --step 1 tests/S288c/genome.fa.gz -o tests/S288c/I.gc.tsv

Rscript templates/peak.tera.R \
    --lag 1000 \
    --influence 20 \
    --threshold 3 \
    --infile tests/S288c/I.gc.tsv \
    --outfile tests/S288c/I.tsv

tsv-summarize tests/S288c/I.tsv \
    -H --group-by signal --count
#signal  count
#0       229884
#-1      142
#1       93

tsv-filter tests/S288c/I.tsv -H --ne signal:0 |
    cut -f 1 |
    linkr merge -c 0.8 stdin | cut -f 2


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

```

* benchmarks

```shell script
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env

hyperfine --warmup 1 --export-markdown garr.md.tmp \
    'garr status drop; garr gen genome.fa.gz --piece 500000' \
    'garr status drop; garr gen genome.fa.gz --piece 500000; garr pos ../TDNA/T-DNA.CSHL.pos.txt'

cat garr.md.tmp

```

| Command          |       Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-----------------|----------------:|---------:|---------:|-------------:|
| `drop; gen`      |    977.6 ± 63.8 |    935.1 |   1153.0 |         1.00 |
| `drop; gen; pos` | 11483.9 ± 242.2 |  11126.4 |  11873.0 | 11.75 ± 0.81 |

