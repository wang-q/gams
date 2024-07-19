# Scer

## genome

```shell
mkdir -p ~/data/gams/Scer/genome
cd ~/data/gams/Scer/genome

# download
wget -N https://ftp.ensemblgenomes.ebi.ac.uk/pub/fungi/release-58/fasta/saccharomyces_cerevisiae/dna/Saccharomyces_cerevisiae.R64-1-1.dna_sm.toplevel.fa.gz

wget -N https://ftp.ensemblgenomes.ebi.ac.uk/pub/fungi/release-58/gff3/saccharomyces_cerevisiae/Saccharomyces_cerevisiae.R64-1-1.58.gff3.gz

# the order of chr in the file is right
gzip -dcf *dna_sm.toplevel* |
    bgzip > genome.fa.gz
faops size genome.fa.gz > chr.sizes

# annotations
gzip -dcf Saccharomyces_cerevisiae.R64-1-1.58.gff3.gz |
    grep -v '^#' |
    cut -f 1 |
    sort | uniq -c
#    504 I
#   1946 II
#    809 III
#   3565 IV
#   1042 IX
#    257 Mito
#   1422 V
#    627 VI
#   2553 VII
#   1383 VIII
#   1734 X
#   1485 XI
#   2559 XII
#   2212 XIII
#   1855 XIV
#   2538 XV
#   2204 XVI

gzip -dcf Saccharomyces_cerevisiae.R64-1-1.58.gff3.gz |
    grep -v '^#' |
    cut -f 3 |
    sort | uniq -c
#   6913 CDS
#     17 chromosome
#   7507 exon
#      4 five_prime_UTR
#   6600 gene
#   6600 mRNA
#     18 ncRNA
#    424 ncRNA_gene
#     12 pseudogene
#     12 pseudogenic_transcript
#     24 rRNA
#      6 snRNA
#     77 snoRNA
#    299 tRNA
#     91 transposable_element
#     91 transposable_element_gene

spanr gff Saccharomyces_cerevisiae.R64-1-1.58.gff3.gz --tag CDS -o cds.json

faops masked genome.fa.gz |
    spanr cover stdin -o repeats.json

spanr merge repeats.json cds.json -o anno.json

spanr stat chr.sizes anno.json --all
#key,chrLength,size,coverage
#cds,12157105,8596583,0.7071
#repeats,12157105,778989,0.0641

```

## vcf

vcf files

[SGRP](http://www.moseslab.csb.utoronto.ca/sgrp/)

```shell
mkdir -p ~/data/gams/Scer/features/
cd ~/data/gams/Scer/features/

wget -N http://www.moseslab.csb.utoronto.ca/sgrp/data/SGRP2-cerevisiae-freebayes-snps-Q30-GQ30.vcf.gz

gzip -dcf SGRP2-cerevisiae-freebayes-snps-Q30-GQ30.vcf.gz |
    bgzip > SGRP2.vcf.gz
bcftools index SGRP2.vcf.gz

# info columns
bcftools view -h SGRP2.vcf.gz |
    perl -nl -e '/INFO=<ID=([\w+.]+),/ and print $1' |
    parallel -j 1 '
        printf "{}\t"
        bcftools query -f '\''%INFO/{}\n'\'' SK1.vcf | tsv-summarize --unique-count 1
    '

bcftools view SGRP2.vcf.gz --samples SK1 --min-alleles 2 --max-alleles 2 --targets "chr1" |
    bcftools annotate --rename-chrs <(echo "chr1 I") |
    bcftools view -i 'GT[0]="A"' `# SK1 has the ALT genotype` |
    bcftools annotate -x "^INFO/AC,INFO/AN,INFO/AF" `# remove useless INFO` |
    bcftools annotate -x "FORMAT/GQ,FORMAT/GL,FORMAT/GLE,FORMAT/QR,FORMAT/QA" `# remove useless FORMAT` -o SK1.vcf

```
