# Atha

## genome

```shell script
mkdir -p ~/data/gars/Atha/genome
cd ~/data/gars/Atha/genome

# download
aria2c -j 4 -x 4 -s 2 -c --file-allocation=none \
    http://ftp.ensemblgenomes.org/pub/release-52/plants/fasta/arabidopsis_thaliana/dna/Arabidopsis_thaliana.TAIR10.dna_sm.toplevel.fa.gz

aria2c -j 4 -x 4 -s 2 -c --file-allocation=none \
    http://ftp.ensemblgenomes.org/pub/release-52/plants/gff3/arabidopsis_thaliana/Arabidopsis_thaliana.TAIR10.52.gff3.gz

# chromosomes
gzip -dcf *dna_sm.toplevel* |
    faops order stdin <(for chr in $(seq 1 1 5) Mt Pt; do echo $chr; done) stdout |
    pigz > genome.fa.gz
faops size genome.fa.gz > chr.sizes

# annotaions
gzip -dcf Arabidopsis_thaliana.TAIR10.52.gff3.gz |
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

gzip -dcf Arabidopsis_thaliana.TAIR10.52.gff3.gz |
    grep -v '^#' |
    cut -f 3 |
    sort | uniq -c
# 286067 CDS
#      7 chromosome
# 313952 exon
#  56384 five_prime_UTR
#  27655 gene
#   3879 lnc_RNA
#    325 miRNA
#  48359 mRNA
#    377 ncRNA
#   5178 ncRNA_gene
#     15 rRNA
#    287 snoRNA
#     82 snRNA
#  48308 three_prime_UTR
#    689 tRNA

spanr gff Arabidopsis_thaliana.TAIR10.52.gff3.gz --tag CDS -o cds.yml

faops masked genome.fa.gz |
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
mkdir -p ~/data/gars/Atha/features/
cd ~/data/gars/Atha/features/

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

## `gars`

### Contigs

```shell script
# start redis-server
rm ~/data/gars/Atha/dump.rdb
redis-server --appendonly no --dir ~/data/gars/Atha/

cd ~/data/gars/Atha/

gars env --all

gars status drop

time gars gen genome/genome.fa.gz --piece 500000
#real    0m1.219s
#user    0m0.980s
#sys     0m0.123s

gars status dump && sync dump.rdb && cp dump.rdb dumps/ctg.dump.rdb

# tsv exports
gars tsv -s 'ctg:*' -f length | head

gars tsv -s 'ctg:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/ctg.tsv

cat tsvs/ctg.tsv |
    sed '1d' |
    cut -f 1 \
    > ctg.lst

# positions
time parallel -j 4 -k --line-buffer '
    echo {}
    gars pos features/T-DNA.{}.pos.txt
    ' ::: CSHL FLAG MX RATM
#real    0m3.663s
#user    0m2.096s
#sys     0m2.666s

gars tsv -s 'pos:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/pos.tsv

gars status dump && sync dump.rdb && cp dump.rdb dumps/pos.dump.rdb

# stop the server
gars status stop

```

### Ranges and rsw

```shell script
cd ~/data/gars/Atha/

rm ./dump.rdb
redis-server --appendonly no --dir ~/data/gars/Atha/
#keydb-server --appendonly no --dir ~/data/gars/Atha/
# keydb is as fast/slow as redis

gars env

gars status drop

gars gen genome/genome.fa.gz --piece 500000

time parallel -j 4 -k --line-buffer '
    echo {}
    gars range features/T-DNA.{}.pos.txt --tag {}
    ' ::: CSHL FLAG MX RATM
# redis
# CSHL
#real    0m1.985s
#user    0m0.344s
#sys     0m1.049s
# 4 files -j 1
#real    0m9.261s
#user    0m1.240s
#sys     0m4.966s
# 4 files -j 4
#real    0m3.824s
#user    0m2.606s
#sys     0m2.362s

gars tsv -s "range:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/range.tsv

gars status dump && sync dump.rdb && cp dump.rdb dumps/range.dump.rdb

# rsw
time cat ctg.lst |
    parallel -j 1 -k --line-buffer '
        gars rsw --ctg {}
        ' |
    tsv-uniq |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/rsw.tsv
# CSHL
# -j 1 redis-server CPU usage at ~70%
# -j 8
#real    0m17.129s
#user    0m21.602s
#sys     0m16.187s
# -j 4
#real    0m18.409s
#user    0m20.197s
#sys     0m15.143s
# -j 2
#real    0m24.126s
#user    0m17.670s
#sys     0m17.428s
# -j 1
#real    0m56.005s
#user    0m15.893s
#sys     0m28.647s

gars status stop

```

### GC-wave

Restores from ctg.dump.rdb

```shell script
cd ~/data/gars/Atha/

cp dumps/ctg.dump.rdb ./dump.rdb
redis-server --appendonly no --dir ~/data/gars/Atha/

gars env

time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        gars sliding \
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

time gars wave tsvs/peak.tsv
#real    4m27.902s
#user    0m26.255s
#sys     2m31.036s

gars tsv -s "peak:*" |
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

gars status stop


```

## clickhouse

* server

```shell script
cd ~/data/gars/Atha/

mkdir -p clickhouse
cd clickhouse
clickhouse server

```

* load

```shell script
cd ~/data/gars/Atha/

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
cd ~/data/gars/Atha/

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
cd ~/data/gars/Atha/

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

