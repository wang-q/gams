# Atha

## genome

```shell script
mkdir -p ~/data/garr/Atha/genome
cd ~/data/garr/Atha/genome

# download
aria2c -j 4 -x 4 -s 2 -c --file-allocation=none \
    ftp://ftp.ensemblgenomes.org/pub/release-45/plants/fasta/arabidopsis_thaliana/dna/Arabidopsis_thaliana.TAIR10.dna_sm.toplevel.fa.gz

aria2c -j 4 -x 4 -s 2 -c --file-allocation=none \
    ftp://ftp.ensemblgenomes.org/pub/release-45/plants/gff3/arabidopsis_thaliana/Arabidopsis_thaliana.TAIR10.45.gff3.gz

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

# .fa.gz
cat {1..5}.fa Mt.fa Pt.fa |
    gzip -9 \
    > genome.fa.gz
faops size genome.fa.gz > chr.sizes

# annotaions
gzip -dcf Arabidopsis_thaliana.TAIR10.45.gff3.gz |
    grep -v '^#' |
    cut -f 1 |
    sort | uniq -c
# 213207 1
# 122450 2
# 152568 3
# 121665 4
# 180531 5
#    615 Mt
#    528 Pt

gzip -dcf Arabidopsis_thaliana.TAIR10.45.gff3.gz |
    grep -v '^#' |
    cut -f 3 |
    sort | uniq -c
# 286067 CDS
#      7 chromosome
# 313952 exon
#  56384 five_prime_UTR
#  27655 gene
#   3879 lnc_RNA
#  48359 mRNA
#    325 miRNA
#    377 ncRNA
#   5178 ncRNA_gene
#     15 rRNA
#     82 snRNA
#    287 snoRNA
#    689 tRNA
#  48308 three_prime_UTR

spanr gff Arabidopsis_thaliana.TAIR10.45.gff3.gz --tag CDS -o cds.yml

faops masked *.fa |
    spanr cover stdin -o repeats.yml

spanr merge repeats.yml cds.yml -o anno.yml
rm repeats.yml cds.yml

spanr stat chr.sizes anno.yml --all
#key,chrLength,size,coverage
#cds,119667750,33775569,0.2822
#repeats,119667750,28237829,0.2360

```

## T-DNA

```shell script
mkdir -p ~/data/garr/Atha/features/
cd ~/data/garr/Atha/features/

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

## `garr`

### Contigs

```shell script
# start redis-server
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env --all

garr status drop

garr gen genome/genome.fa.gz --piece 500000

# redis dumps
mkdir -p ~/data/garr/Atha/dumps/

while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        cp dump.rdb dumps/ctg.dump.rdb
        break
    fi
    sleep 5
done

# tsv exports
mkdir -p tsvs

garr tsv -s 'ctg:*' -f length | head

garr tsv -s 'ctg:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/ctg.tsv

cat tsvs/ctg.tsv |
    sed '1d' |
    cut -f 1 \
    > ctg.lst

# positions
parallel -j 4 -k --line-buffer '
    echo {}
    garr pos features/T-DNA.{}.pos.txt
    ' ::: CSHL FLAG MX RATM

garr tsv -s 'pos:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/pos.tsv

# dumps
while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        mkdir -p dumps
        cp dump.rdb dumps/pos.dump.rdb
        break
    fi
    sleep 5
done

# stop the server
garr status stop


```

### Ranges and rsw

* Benchmarks keydb against redis

```shell script
cd ~/data/garr/Atha/

rm ./dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/
#keydb-server --appendonly no --dir ~/data/garr/Atha/
# keydb is as fast/slow as redis

garr env

garr status drop

time garr gen genome/genome.fa.gz --piece 500000
#real    0m1.520s
#user    0m0.582s
#sys     0m0.407s

time parallel -j 4 -k --line-buffer '
    echo {}
    garr range features/T-DNA.{}.pos.txt --tag {}
    ' ::: CSHL FLAG MX RATM
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

garr tsv -s "range:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/range.tsv

while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        mkdir -p dumps
        cp dump.rdb dumps/range.dump.rdb
        break
    fi
    sleep 5
done

# rsw
time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        garr rsw --ctg {}
        ' |
    tsv-uniq |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/rsw.tsv
# CSHL
# -j 4
#real    7m43.384s
#user    24m58.916s
#sys     3m42.415s
# -j 2
#real    13m38.805s
#user    21m56.417s
#sys     4m34.154s

garr status stop

```

* Benchmarks  against redis under WSL

```shell
# start redis-server
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

# benchmarks
garr env

garr status drop

time garr gen genome/genome.fa.gz --piece 500000
#real    0m0.771s
#user    0m0.611s
#sys     0m0.110s

time parallel -j 4 -k --line-buffer '
    echo {}
    garr range features/T-DNA.{}.pos.txt --tag {}
    ' ::: CSHL
#real    0m9.462s
#user    0m1.271s
#sys     0m4.027s

time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        garr rsw --ctg {}
        ' |
    tsv-uniq |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/rsw.tsv
#real    2m47.957s
#user    2m46.630s
#sys     2m20.460s

garr status stop

```

* docker with NTFS

```shell

# start redis from docker and bring local dir into it
docker run -p 6379:6379 -v C:/Users/wangq/data/garr/Atha:/data redislabs/redisearch:latest

cp /usr/lib/redis/modules/redisearch.so .

cd /mnt/c/Users/wangq/data/garr/Atha

```

* with `redisearch.so`

```shell
# copy `redisearch.so` from the docker image
docker run -it --rm --entrypoint /bin/sh -v C:/Users/wangq/data/garr/Atha:/data redislabs/redisearch

# start redis-server
redis-server --loadmodule ./redisearch.so --appendonly no --dir ~/data/garr/Atha/

```

| Command           |      gen |     range |         rsw |
|:------------------|---------:|----------:|------------:|
| WSL               | 0m0.771s |  0m9.462s |   2m47.957s |
| redisearch.so     | 0m0.803s |  0m9.929s |   2m53.232s |
| docker under NTFS | 0m1.440s | 0m41.657s |   12m3.494s |


### GC-wave

Restores from ctg.dump.rdb

```shell script
cd ~/data/garr/Atha/

cp dumps/ctg.dump.rdb ./dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/

garr env

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
            > tsvs/{.}.peak.tsv

        tsv-summarize tsvs/{.}.peak.tsv \
            -H --group-by signal --count

        rm {.}.gc.tsv {.}.replace.tsv
    '
#real    5m58.731s
#user    23m47.703s
#sys     0m16.114s

# Don't need to be sorted
tsv-append $(cat ctg.lst | sed 's/$/.peak.tsv/' | sed 's/^/tsvs\//') -H \
    > tsvs/peak.tsv

rm tsvs/ctg:*.peak.tsv

tsv-summarize tsvs/peak.tsv \
    -H --group-by signal --count
#signal  count
#1       32361
#-1      26944

time garr wave tsvs/peak.tsv
#real    4m27.902s
#user    0m26.255s
#sys     2m31.036s

garr tsv -s "peak:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/wave.tsv

cat tsvs/wave.tsv |
    tsv-summarize -H --count
# 59305

cat tsvs/wave.tsv |
    tsv-filter -H --gt left_wave_length:0 |
    tsv-summarize -H --mean left_wave_length

cat tsvs/wave.tsv |
    tsv-filter -H --gt right_wave_length:0 |
    tsv-summarize -H --mean right_wave_length

tsv-filter tsvs/wave.tsv -H --or \
    --le left_wave_length:0 --le right_wave_length:0 |
    tsv-summarize -H --count
# 13003

garr status stop


```

## Benchmarks

```shell script
cd ~/data/garr/Atha/

rm ./dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/

garr env

hyperfine --warmup 1 --export-markdown garr.md.tmp \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr pos features/T-DNA.CSHL.pos.txt;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr range features/T-DNA.CSHL.pos.txt --tag CSHL;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr sliding --size 100 --step 20 --lag 50 |
            tsv-filter -H --ne signal:0 > /dev/null;
    '

cat garr.md.tmp

```

* R5 4600U

| Command               |       Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:----------------------|----------------:|---------:|---------:|-------------:|
| `drop; gen;`          |    813.8 ± 18.5 |    783.4 |    834.4 |         1.00 |
| `drop; gen; pos;`     |  8203.5 ± 166.7 |   8051.7 |   8475.7 | 10.08 ± 0.31 |
| `drop; gen; range;`   | 20045.7 ± 474.6 |  19218.6 |  21041.7 | 24.63 ± 0.81 |
| `drop; gen; sliding;` |   7580.7 ± 72.6 |   7467.1 |   7705.8 |  9.31 ± 0.23 |

* R7 5800

| Command               |       Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:----------------------|----------------:|---------:|---------:|-------------:|
| `drop; gen;`          |     788.0 ± 3.8 |    783.8 |    794.0 |         1.00 |
| `drop; gen; pos;`     |   4840.2 ± 47.6 |   4764.4 |   4917.1 |  6.14 ± 0.07 |
| `drop; gen; range;`   | 10872.3 ± 105.0 |  10753.9 |  11119.2 | 13.80 ± 0.15 |
| `drop; gen; sliding;` |   5031.6 ± 24.0 |   4996.0 |   5071.1 |  6.38 ± 0.04 |

## clickhouse

* server

```shell script
cd ~/data/garr/Atha/

mkdir -p clickhouse
cd clickhouse
clickhouse server

```

* load

```shell script
cd ~/data/garr/Atha/

for q in ctg rsw; do
    clickhouse client --query "DROP TABLE IF EXISTS ${q}"
    clickhouse client --query "$(cat sqls/ddl/${q}.sql)"
done

for q in ctg rsw; do
    echo ${q}
    cat tsvs/${q}.tsv |
        clickhouse client --query "INSERT INTO ${q} FORMAT TSVWithNames"
done

```

* queries

```shell script
cd ~/data/garr/Atha/

mkdir -p stats

# summary
ARRAY=(
    'ctg::length'
    'rsw::gc_content'
)

for item in "${ARRAY[@]}"; do
    echo ${item} 1>&2
    TABLE="${item%%::*}"
    COLUMN="${item##*::}"

    clickhouse client --query "$(
        cat sqls/summary.sql | sed "s/_TABLE_/${TABLE}/" | sed "s/_COLUMN_/${COLUMN}/"
    )"
done |
    tsv-uniq \
    > stats/summary.tsv

for t in rsw; do
    echo ${t} 1>&2
    clickhouse client --query "$(cat sqls/summary-type.sql | sed "s/_TABLE_/${t}/")"
done |
    tsv-uniq \
    > stats/summary-type.tsv

# rsw
for q in rsw-distance rsw-distance-tag; do
    echo ${q}
    clickhouse client --query "$(cat sqls/${q}.sql)" > stats/${q}.tsv
done

```

## plots

* rsw-distance-tag

```shell script
cd ~/data/garr/Atha/

mkdir -p plots

cat stats/rsw-distance-tag.tsv |
    cut -f 1 |
    grep -v "^tag$" |
    tsv-uniq \
    > plots/tag.lst

for tag in $(cat plots/tag.lst); do
    echo ${tag}
    base="rsw-distance-tag.${tag}"

    cat stats/rsw-distance-tag.tsv |
        tsv-filter -H --str-eq tag:${tag} |
        tsv-select -H --exclude tag \
        > plots/${base}.tsv

    for y in {2..6}; do
        echo ${y}
        Rscript plot_xy.R --infile plots/${base}.tsv --ycol ${y} --yacc 0.002 --outfile plots/${base}.${y}.pdf
    done

    gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite -sOutputFile=plots/${base}.pdf \
        $( for y in {2..6}; do echo plots/${base}.${y}.pdf; done )

    for y in {2..6}; do
        rm plots/${base}.${y}.pdf
    done

    pdfjam plots/${base}.pdf --nup 5x1 --suffix nup -o plots

    pdfcrop plots/${base}-nup.pdf
    mv plots/${base}-nup-crop.pdf plots/${base}-nup.pdf

    rm plots/${base}.tsv
done

```

