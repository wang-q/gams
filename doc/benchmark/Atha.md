# Atha

## data

Avoid HDD

```shell
mkdir -p ~/gars
cd ~/gars

cp ~/data/gars/Atha/genome/genome.fa.gz .
cp ~/data/gars/Atha/features/T-DNA.CSHL.pos.txt .

```

## hyperfine

```shell
cd ~/gars

# redis
rm dump.rdb
redis-server

# gars
gars env

hyperfine --warmup 1 --export-markdown gars.md.tmp \
    '
        gars status drop;
        gars gen genome.fa.gz --piece 500000;
    ' \
    '
        gars status drop;
        gars gen genome.fa.gz --piece 500000;
        gars pos T-DNA.CSHL.pos.txt;
    ' \
    '
        gars status drop;
        gars gen genome.fa.gz --piece 500000;
        gars range T-DNA.CSHL.pos.txt --tag CSHL;
    ' \
    '
        gars status drop;
        gars gen genome.fa.gz --piece 500000;
        gars sliding --size 100 --step 20 --lag 50 |
            tsv-filter -H --ne signal:0 > /dev/null;
    '

cat gars.md.tmp

```

## R7 5800 Windows 11

| Command             |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------|--------------:|--------:|--------:|------------:|
| drop; gen;          | 1.082 ± 0.002 |   1.079 |   1.086 |        1.00 |
| drop; gen; pos;     | 5.060 ± 0.113 |   4.944 |   5.252 | 4.68 ± 0.11 |
| drop; gen; range;   | 5.037 ± 0.068 |   4.975 |   5.175 | 4.66 ± 0.06 |
| drop; gen; sliding; | 5.761 ± 0.017 |   5.730 |   5.780 | 5.33 ± 0.02 |
