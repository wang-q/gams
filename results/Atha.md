# Atha

## genome

```shell
mkdir -p ~/data/gams/Atha/genome
cd ~/data/gams/Atha/genome

# download
wget -N https://ftp.ensemblgenomes.ebi.ac.uk/pub/plants/release-58/fasta/arabidopsis_thaliana/dna/Arabidopsis_thaliana.TAIR10.dna_sm.toplevel.fa.gz

wget -N https://ftp.ensemblgenomes.ebi.ac.uk/pub/plants/release-58/gff3/arabidopsis_thaliana/Arabidopsis_thaliana.TAIR10.58.gff3.gz

# chromosomes
gzip -dcf *dna_sm.toplevel* |
    faops order stdin <(for chr in $(seq 1 1 5) Mt Pt; do echo $chr; done) stdout |
    pigz > genome.fa.gz
faops size genome.fa.gz > chr.sizes

# annotaions
gzip -dcf Arabidopsis_thaliana.TAIR10.58.gff3.gz |
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

gzip -dcf Arabidopsis_thaliana.TAIR10.58.gff3.gz |
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

spanr gff Arabidopsis_thaliana.TAIR10.58.gff3.gz --tag CDS -o cds.json

faops masked genome.fa.gz |
    spanr cover stdin -o repeats.json

spanr merge repeats.json cds.json -o anno.json

spanr stat chr.sizes anno.json --all
#key,chrLength,size,coverage
#cds,119667750,33775569,0.2822
#repeats,119667750,38274794,0.3198

```

## T-DNA

```shell
mkdir -p ~/data/gams/Atha/features/
cd ~/data/gams/Atha/features/

for name in CSHL FLAG MX RATM; do
    aria2c -j 4 -x 4 -s 2 --file-allocation=none -c \
        http://natural.salk.edu/database/tdnaexpress/T-DNA.${name}
done

# Convert to ranges
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
        > T-DNA.${name}.rg;
done

```

## `gams`

### Contigs

```shell
cd ~/data/gams/Atha/

# start redis-server
redis-server &
gams env --all

gams status drop

gams gen genome/genome.fa.gz --piece 500000

gams status dump dumps/ctg.rdb

# tsv exports
time gams tsv -s 'ctg:*' |
    gams anno -H genome/cds.json stdin |
    gams anno -H genome/repeats.json stdin |
    rgr sort -H -f 2 stdin |
    tsv-select -H -e range |
    pigz \
    > tsvs/ctg.tsv.gz
#real    0m0.977s
#user    0m1.001s
#sys     0m0.049s

gzip -dcf tsvs/ctg.tsv.gz |
    sed '1d' |
    cut -f 1 \
    > ctg.lst

# ranges
time parallel -j 4 -k --line-buffer '
    echo {}
    gams range features/T-DNA.{}.rg
    ' ::: CSHL FLAG MX RATM
#real    0m1.086s
#user    0m0.423s
#sys     0m0.225s

time gams tsv -s 'range:*' |
    gams anno genome/cds.json stdin -H |
    gams anno genome/repeats.json stdin -H |
    rgr sort -H -f 2 stdin |
    tsv-select -H -e range |
    pigz \
    > tsvs/range.tsv.gz
#real    0m7.933s
#user    0m4.481s
#sys     0m4.459s

gams status dump dumps/range.rdb

# stop the server
gams status stop

```

### Features and fsw

```shell
cd ~/data/gams/Atha/

rm ./dump.rdb
redis-server --appendonly no --dir ~/data/gams/Atha/

gams env --all

gams status drop

gams gen genome/genome.fa.gz --piece 500000

parallel -j 4 -k --line-buffer '
    echo {}
    gams feature features/T-DNA.{}.rg --tag {}
    ' ::: CSHL FLAG MX RATM

time gams tsv -s 'feature:*' |
    gams anno genome/cds.json stdin -H |
    gams anno genome/repeats.json stdin -H |
    rgr sort -H -f 2 stdin |
    tsv-select -H -e range |
    pigz \
    > tsvs/feature.tsv.gz
#real    0m8.440s
#user    0m5.084s
#sys     0m4.266s

gams status dump && sync dump.rdb && cp dump.rdb dumps/feature.dump.rdb

# fsw
time gams fsw --range |
    rgr sort -H -f 2 stdin |
    pigz \
    > tsvs/fsw.tsv.gz
#real    0m33.137s
#user    0m30.607s
#sys     0m6.326s

time cat genome/chr.sizes |
    cut -f 1 |
    parallel -j 4 -k --line-buffer '
        gams fsw --ctg "ctg:{}:*" --range |
            gams anno genome/cds.json stdin -H |
            gams anno genome/repeats.json stdin -H |
            rgr sort -H -f 2 stdin
        ' |
    rgr sort -H -f 2 stdin |
    tsv-select -H -e range |
    tsv-uniq |
    pigz \
    > tsvs/fsw.tsv.gz
#real    0m34.185s
#user    2m52.237s
#sys     0m9.119s

gams status stop

```

### GC-wave

Restores from ctg.dump.rdb

```shell
cd ~/data/gams/Atha/

cp dumps/ctg.rdb ./dump.rdb
redis-server &

gams env

# split a chr to 10
cat ctg.lst |
    perl -nl -e '
        BEGIN {%seen}
        s/(ctg:\w+:\d)\d*$/$1/;
        $seen{"$1*"}++;
        END { print for sort keys %seen}
    ' \
    > ctg.group.lst

# can't use chr.sizes, which greatly reduces the speed of `rgr merge`
time cat ctg.group.lst |
    parallel -j 4 -k --line-buffer '
        prefix=$(echo {} | sed "s/[^[:alnum:]-]/_/g")
        export prefix

        gams sliding \
            --ctg {} \
            --size 100 --step 1 \
            --lag 1000 --threshold 3.0 --influence 1.0 \
            --parallel 4 \
            -o stdout |
            tsv-filter -H --ne signal:0 |
            rgr sort -H stdin |
            rgr pl-2rmp -c 0.8 stdin |
            tsv-uniq -H -f 1 \
            > tsvs/${prefix}.peak.tsv

        tsv-summarize tsvs/${prefix}.peak.tsv \
            -H --group-by signal --count
    '
#real    1m23.736s
#user    10m3.180s
#sys     0m16.014s

# Don't need to be sorted
tsv-append -H $(
    cat ctg.group.lst |
        sed "s/[^[:alnum:]-]/_/g" `# Remove : *` |
        sed 's/$/.peak.tsv/' `# suffix` |
        sed 's/^/tsvs\//' `# dir`
    ) \
    > tsvs/peak.tsv

rm tsvs/ctg*.peak.tsv

tsv-summarize tsvs/peak.tsv \
    -H --group-by signal --count
#signal  count
#1       32924
#-1      27371

# Loading peaks
time gams peak tsvs/peak.tsv
#real    0m23.482s
#user    0m5.295s
#sys     0m13.237s

gams tsv -s "peak:*" |
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

gams status stop

```

## clickhouse

* server

```shell
cd ~/data/gams/Atha/

mkdir -p clickhouse
cd clickhouse
clickhouse server

```

* load

```shell
cd ~/data/gams/Atha/

for q in ctg fsw; do
    clickhouse client --query "DROP TABLE IF EXISTS ${q}"
    clickhouse client --query "$(cat sqls/ddl/${q}.sql)"
done

for q in ctg fsw; do
    echo ${q}
    gzip -dcf tsvs/${q}.tsv.gz |
        clickhouse client --query "INSERT INTO ${q} FORMAT TSVWithNames"
done

```

* queries

```shell
cd ~/data/gams/Atha/

mkdir -p stats

# summary
ARRAY=(
    'ctg::length'
    'fsw::gc_content'
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

for t in fsw; do
    echo ${t} 1>&2
    clickhouse client --query "$(cat sqls/summary-type.sql | sed "s/_TABLE_/${t}/")"
done |
    tsv-uniq \
    > stats/summary-type.tsv

# fsw
for q in fsw-distance fsw-distance-tag; do
    echo ${q}
    clickhouse client --query "$(cat sqls/${q}.sql)" > stats/${q}.tsv
done

```

## plots

### fsw-distance-tag

```shell
cd ~/data/gams/Atha/

mkdir -p plots

cat stats/fsw-distance-tag.tsv |
    cut -f 1 |
    grep -v "^tag$" |
    tsv-uniq \
    > plots/tag.lst

for tag in $(cat plots/tag.lst); do
    echo ${tag}
    base="fsw-distance-tag.${tag}"

    cat stats/fsw-distance-tag.tsv |
        tsv-filter -H --str-eq tag:${tag} |
        tsv-select -H --exclude tag \
        > plots/${base}.tsv

    for y in {2..7}; do
        echo ${y}
        Rscript plot_xy.R --infile plots/${base}.tsv --ycol ${y} --yacc 0.002 --outfile plots/${base}.${y}.pdf
    done

    gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite -sOutputFile=plots/${base}.pdf \
        $( for y in {2..7}; do echo plots/${base}.${y}.pdf; done )

    for y in {2..7}; do
        rm plots/${base}.${y}.pdf
    done

    pdfjam plots/${base}.pdf --nup 7x1 --suffix nup -o plots

    pdfcrop --margins 5 plots/${base}-nup.pdf
    mv plots/${base}-nup-crop.pdf plots/${base}-nup.pdf

    rm plots/${base}.tsv
done

#gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite -sOutputFile=plots/fsw-distance-tag.pdf \
#    $( for tag in $(cat plots/tag.lst); do echo plots/fsw-distance-tag.${tag}-nup.pdf; done )
#
#pdfjam plots/fsw-distance-tag.pdf --nup 1x5 --suffix nup -o plots


```

