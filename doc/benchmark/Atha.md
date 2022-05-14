# Atha

## data

Avoid HDD

```shell
mkdir -p ~/gars
cd ~/gars

cp ~/data/gars/Atha/genome/genome.fa.gz .
cp ~/data/gars/Atha/genome/chr.sizes .

cp ~/data/gars/Atha/features/T-DNA.CSHL.rg .
cp ~/data/gars/Atha/features/T-DNA.FLAG.rg .
cp ~/data/gars/Atha/features/T-DNA.MX.rg .
cp ~/data/gars/Atha/features/T-DNA.RATM.rg .

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
    -n 'drop; gen;' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    ' \
    -n 'd-g; range;' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars range T-DNA.CSHL.rg;
    ' \
    -n 'clear; range;' \
    '
    gars clear range;
    gars range T-DNA.CSHL.rg;
    ' \
    -n 'd-g; feature;' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars feature T-DNA.CSHL.rg --tag CSHL;
    ' \
    -n 'd-g; feature; fsw;' \
    '
    gars status drop; gars gen genome.fa.gz --piece 500000;
    gars feature T-DNA.CSHL.rg --tag CSHL;
    gars fsw;
    ' \
    -n 'd-g; sliding;' \
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

| Command              |       Mean [s] | Min [s] | Max [s] |     Relative |
|:---------------------|---------------:|--------:|--------:|-------------:|
| `drop; gen;`         |  1.664 ± 0.044 |   1.629 |   1.782 |         1.00 |
| `d-g; range;`        |  3.438 ± 0.156 |   3.328 |   3.746 |  2.07 ± 0.11 |
| `clear; range;`      |  2.524 ± 0.065 |   2.459 |   2.622 |  1.52 ± 0.06 |
| `d-g; feature;`      |  3.775 ± 0.195 |   3.513 |   4.154 |  2.27 ± 0.13 |
| `d-g; feature; fsw;` | 18.547 ± 0.780 |  17.982 |  20.004 | 11.15 ± 0.55 |
| `d-g; sliding;`      | 11.118 ± 0.262 |  10.859 |  11.742 |  6.68 ± 0.24 |

## threads

```shell
# redis
cd ~/gars
rm dump.rdb
redis-server

# gars
cd ~/gars
gars env

# feature
gars status drop; gars gen genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars clear feature;
    gars feature T-DNA.CSHL.rg --tag CSHL;
    gars feature T-DNA.FLAG.rg --tag FLAG;
    gars feature T-DNA.MX.rg   --tag MX;
    gars feature T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gars clear feature;
    parallel -j 1 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gars clear feature;
    parallel -j 2 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gars clear feature;
    parallel -j 4 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

# fsw
gars status drop; gars gen genome.fa.gz --piece 500000;

gars feature T-DNA.CSHL.rg --tag CSHL;
gars feature T-DNA.FLAG.rg --tag FLAG;
gars feature T-DNA.MX.rg   --tag MX;
gars feature T-DNA.RATM.rg --tag RATM;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars fsw > /dev/null;
    ' \
    -n 'parallel -j 1' \
    '
    cat chr.sizes |
        cut -f 1 |
        parallel -j 1 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    ' \
    -n 'parallel -j 2' \
    '
    cat chr.sizes |
        cut -f 1 |
        parallel -j 2 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    ' \
    -n 'parallel -j 4' \
    '
    cat chr.sizes |
        cut -f 1 |
        parallel -j 4 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    '

cat threads.md.tmp

```

### i7 8700K macOS Big Sur

* feature

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 12.456 ± 1.254 |  11.814 |  15.908 | 1.61 ± 0.17 |
| `parallel -j 1` | 13.276 ± 1.253 |  12.492 |  16.573 | 1.72 ± 0.17 |
| `parallel -j 2` |  8.980 ± 0.226 |   8.677 |   9.424 | 1.16 ± 0.04 |
| `parallel -j 4` |  7.735 ± 0.188 |   7.359 |   7.992 |        1.00 |

* fsw

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 59.416 ± 0.226 |  59.159 |  59.762 | 2.58 ± 0.07 |
| `parallel -j 1` | 65.700 ± 0.241 |  65.444 |  66.138 | 2.85 ± 0.08 |
| `parallel -j 2` | 34.759 ± 0.087 |  34.648 |  34.930 | 1.51 ± 0.04 |
| `parallel -j 4` | 23.027 ± 0.615 |  22.263 |  24.429 |        1.00 |

## keydb

keydb is as fast/slow as redis

```shell
# brew install keydb
cd ~/gars
rm dump.rdb
keydb-server

# gars
cd ~/gars
gars env

gars status drop; gars gen genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars clear feature;
    gars feature T-DNA.CSHL.rg --tag CSHL;
    gars feature T-DNA.FLAG.rg --tag FLAG;
    gars feature T-DNA.MX.rg   --tag MX;
    gars feature T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gars clear feature;
    parallel -j 1 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gars clear feature;
    parallel -j 2 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gars clear feature;
    parallel -j 4 "
        echo {}
        gars feature T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

```

### i7 8700K macOS Big Sur

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 14.911 ± 0.271 |  14.646 |  15.370 | 1.60 ± 0.06 |
| `parallel -j 1` | 15.783 ± 0.486 |  15.088 |  16.607 | 1.69 ± 0.07 |
| `parallel -j 2` | 10.753 ± 0.351 |  10.438 |  11.437 | 1.15 ± 0.05 |
| `parallel -j 4` |  9.342 ± 0.296 |   8.994 |   9.795 |        1.00 |

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
    -n 'idx' \
    'gars locate -f T-DNA.CSHL.rg' \
    -n 'lapper' \
    'gars locate --lapper -f T-DNA.CSHL.rg' \
    -n 'zrange' \
    'gars locate --zrange -f T-DNA.CSHL.rg'

cat locate.md.tmp

```

### R7 5800 Windows 11

| Command |     Mean [ms] | Min [ms] | Max [ms] |      Relative |
|:--------|--------------:|---------:|---------:|--------------:|
| idx     |    22.8 ± 3.1 |     21.7 |     50.9 |          1.00 |
| lapper  |  917.6 ± 16.6 |    892.2 |    938.9 |  40.25 ± 5.44 |
| zrange  | 2064.1 ± 34.0 |   2022.3 |   2136.7 | 90.55 ± 12.21 |

### i7 8700K macOS Big Sur

| Command  |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|--------------:|---------:|---------:|-------------:|
| `idx`    |    62.2 ± 1.1 |     60.0 |     65.2 |         1.00 |
| `lapper` |   959.6 ± 9.4 |    945.4 |    974.2 | 15.43 ± 0.31 |
| `zrange` | 3161.5 ± 64.4 |   3097.3 |   3289.5 | 50.82 ± 1.36 |
