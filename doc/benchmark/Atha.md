# Atha

## data

Avoid HDD

```shell
mkdir -p ~/gars
cd ~/gars

cp ~/data/gars/Atha/genome/genome.fa.gz .
cp ~/data/gars/Atha/features/T-DNA.CSHL.pos.txt .

```

## `gars`

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
        gars sliding --size 100 --step 20 --lag 50 > /dev/null;
    '

cat gars.md.tmp

```

### R7 5800 Windows 11

| Command             |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------|--------------:|--------:|--------:|------------:|
| drop; gen;          | 1.126 ± 0.004 |   1.123 |   1.135 |        1.00 |
| drop; gen; pos;     | 2.995 ± 0.031 |   2.956 |   3.061 | 2.66 ± 0.03 |
| drop; gen; range;   | 3.514 ± 0.681 |   3.015 |   5.095 | 3.12 ± 0.61 |
| drop; gen; sliding; | 5.624 ± 0.024 |   5.603 |   5.685 | 5.00 ± 0.03 |

## `gars locate`

```shell
cd ~/gars

# redis
rm dump.rdb
redis-server

# gars
gars env

gars status drop

gars gen genome.fa.gz --piece 500000;

hyperfine -N --export-markdown locate.md.tmp \
    'gars locate -f T-DNA.CSHL.pos.txt' \
    'gars locate --idx -f T-DNA.CSHL.pos.txt' \
    'gars locate --zrange -f T-DNA.CSHL.pos.txt'

cat locate.md.tmp

```

### R7 5800 Windows 11

| Command |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:--------|--------------:|---------:|---------:|-------------:|
| lapper  |  919.9 ± 19.7 |    890.5 |    946.6 | 41.85 ± 1.22 |
| idx     |    22.0 ± 0.4 |     21.3 |     24.1 |         1.00 |
| zrange  | 2048.7 ± 11.4 |   2037.8 |   2065.4 | 93.21 ± 1.89 |
