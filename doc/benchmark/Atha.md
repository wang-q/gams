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
# redis
cd ~/gars
rm dump.rdb
redis-server

# gars
cd ~/gars
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
        gars range T-DNA.CSHL.pos.txt --tag CSHL;
        gars rsw;
    ' \
    '
        gars status drop;
        gars gen genome.fa.gz --piece 500000;
        gars sliding --size 100 --step 20 --lag 50 > /dev/null;
    '

cat gars.md.tmp

```

### R7 5800 Windows 11

| Command          |      Mean [s] | Min [s] | Max [s] |    Relative |
|:-----------------|--------------:|--------:|--------:|------------:|
| drop; gen;       | 1.104 ± 0.009 |   1.085 |   1.116 |        1.00 |
| d-g; pos;        | 2.949 ± 0.031 |   2.898 |   2.990 | 2.67 ± 0.04 |
| d-g; range;      | 3.418 ± 0.537 |   2.992 |   4.264 | 3.10 ± 0.49 |
| d-g; range; rsw; | 8.878 ± 0.141 |   8.662 |   9.132 | 8.04 ± 0.14 |
| d-g; sliding;    | 5.555 ± 0.016 |   5.530 |   5.582 | 5.03 ± 0.04 |

## `gars locate`

```shell
# redis
cd ~/gars
rm dump.rdb
redis-server

# gars
cd ~/gars
gars env

gars status drop

gars gen genome.fa.gz --piece 500000

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
