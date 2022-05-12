# Atha

## data

Avoid HDD

```shell
mkdir -p ~/gars
cd ~/gars

cp ~/data/gars/Atha/genome/genome.fa.gz .
cp ~/data/gars/Atha/features/T-DNA.CSHL.ranges .

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
    gars status drop; gars gen genome.fa.gz --piece 500000;
    ' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars range T-DNA.CSHL.ranges;
    ' \
    '
    gars clear range;
    gars range T-DNA.CSHL.ranges;
    ' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars feature T-DNA.CSHL.ranges --tag CSHL;
    ' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars feature T-DNA.CSHL.ranges --tag CSHL;
    gars fsw;
    ' \
    '
    gars status drop;gars gen genome.fa.gz --piece 500000;
    gars sliding --size 100 --step 20 --lag 50 > /dev/null;
    '

cat gars.md.tmp

```

### R7 5800 Windows 11

| Command            |      Mean [s] | Min [s] | Max [s] |    Relative |
|:-------------------|--------------:|--------:|--------:|------------:|
| drop; gen;         | 1.102 ± 0.006 |   1.091 |   1.114 |        1.00 |
| d-g; range;        | 2.932 ± 0.036 |   2.895 |   2.997 | 2.66 ± 0.04 |
| clear; range;      | 2.742 ± 0.043 |   2.676 |   2.804 | 2.49 ± 0.04 |
| d-g; feature;      | 3.011 ± 0.029 |   2.965 |   3.044 | 2.73 ± 0.03 |
| d-g; feature; fsw; | 8.817 ± 0.085 |   8.715 |   9.004 | 8.00 ± 0.09 |
| d-g; sliding;      | 5.585 ± 0.035 |   5.553 |   5.673 | 5.07 ± 0.04 |

### i7 8700K macOS Big Sur

| Command            |       Mean [s] | Min [s] | Max [s] |     Relative |
|:-------------------|---------------:|--------:|--------:|-------------:|
| drop; gen;         |  1.713 ± 0.023 |   1.695 |   1.776 |         1.00 |
| d-g; range;        |  3.459 ± 0.051 |   3.394 |   3.523 |  2.02 ± 0.04 |
| clear; range;      |  2.510 ± 0.051 |   2.468 |   2.649 |  1.47 ± 0.04 |
| d-g; feature;      |  3.528 ± 0.048 |   3.488 |   3.638 |  2.06 ± 0.04 |
| d-g; feature; fsw; | 18.214 ± 0.235 |  18.061 |  18.860 | 10.63 ± 0.20 |
| d-g; sliding;      | 10.943 ± 0.119 |  10.823 |  11.146 |  6.39 ± 0.11 |

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
    'gars locate -f T-DNA.CSHL.ranges' \
    'gars locate --idx -f T-DNA.CSHL.ranges' \
    'gars locate --zrange -f T-DNA.CSHL.ranges'

cat locate.md.tmp

```

### R7 5800 Windows 11

| Command |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:--------|--------------:|---------:|---------:|-------------:|
| lapper  |  919.9 ± 19.7 |    890.5 |    946.6 | 41.85 ± 1.22 |
| idx     |    22.0 ± 0.4 |     21.3 |     24.1 |         1.00 |
| zrange  | 2048.7 ± 11.4 |   2037.8 |   2065.4 | 93.21 ± 1.89 |

### i7 8700K macOS Big Sur

| Command |      Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:--------|---------------:|---------:|---------:|-------------:|
| lapper  |   982.3 ± 72.0 |    930.0 |   1153.7 | 15.87 ± 1.19 |
| idx     |     61.9 ± 1.0 |     60.0 |     64.7 |         1.00 |
| zrange  | 3596.2 ± 281.7 |   3205.8 |   4031.0 | 58.11 ± 4.64 |
