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

| Command              |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|--------------:|--------:|--------:|------------:|
| `drop; gen;`         | 1.117 ± 0.004 |   1.109 |   1.124 |        1.00 |
| `d-g; range;`        | 3.024 ± 0.038 |   2.978 |   3.068 | 2.71 ± 0.04 |
| `clear; range;`      | 2.827 ± 0.060 |   2.718 |   2.938 | 2.53 ± 0.05 |
| `d-g; feature;`      | 3.055 ± 0.050 |   2.980 |   3.144 | 2.73 ± 0.05 |
| `d-g; feature; fsw;` | 8.949 ± 0.149 |   8.743 |   9.185 | 8.01 ± 0.14 |
| `d-g; sliding;`      | 5.540 ± 0.028 |   5.496 |   5.587 | 4.96 ± 0.03 |

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

### R7 5800 Windows 11

* feature

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 12.903 ± 0.257 |  12.603 |  13.436 | 1.64 ± 0.04 |
| `parallel -j 1` | 13.285 ± 0.239 |  12.890 |  13.811 | 1.68 ± 0.04 |
| `parallel -j 2` |  8.207 ± 0.245 |   7.961 |   8.854 | 1.04 ± 0.03 |
| `parallel -j 4` |  7.892 ± 0.107 |   7.742 |   8.040 |        1.00 |

* fsw

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 23.775 ± 0.232 |  23.514 |  24.198 | 2.46 ± 0.03 |
| `parallel -j 1` | 28.501 ± 0.549 |  27.699 |  29.672 | 2.94 ± 0.06 |
| `parallel -j 2` | 14.920 ± 0.266 |  14.651 |  15.550 | 1.54 ± 0.03 |
| `parallel -j 4` |  9.684 ± 0.052 |   9.611 |   9.766 |        1.00 |

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

### R7 5800 Windows 11

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 16.082 ± 0.197 |  15.746 |  16.531 | 1.70 ± 0.04 |
| `parallel -j 1` | 16.938 ± 0.246 |  16.335 |  17.178 | 1.80 ± 0.05 |
| `parallel -j 2` |  9.824 ± 0.225 |   9.423 |  10.263 | 1.04 ± 0.03 |
| `parallel -j 4` |  9.435 ± 0.194 |   9.104 |   9.808 |        1.00 |

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

| Command  |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|-------------:|---------:|---------:|-------------:|
| `idx`    |   22.1 ± 0.5 |     21.3 |     25.8 |         1.00 |
| `lapper` | 930.5 ± 14.3 |    907.7 |    957.7 | 42.11 ± 1.23 |
| `zrange` | 2041.1 ± 9.6 |   2029.8 |   2056.4 | 92.37 ± 2.33 |

### i7 8700K macOS Big Sur

| Command  |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|--------------:|---------:|---------:|-------------:|
| `idx`    |    62.2 ± 1.1 |     60.0 |     65.2 |         1.00 |
| `lapper` |   959.6 ± 9.4 |    945.4 |    974.2 | 15.43 ± 0.31 |
| `zrange` | 3161.5 ± 64.4 |   3097.3 |   3289.5 | 50.82 ± 1.36 |

## `rgr`

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

hyperfine --warmup 1 --export-markdown rgr.md.tmp \
    -n 'tsv-sort' \
    '
    gars tsv -s "ctg:*" |
        keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n
    ' \
    -n 'rgr sort' \
    '
    gars tsv -s "ctg:*" --range |
        rgr sort -H -f 2 stdin
    ' \
    -n 'rgr prop' \
    '
    gars tsv -s "ctg:*" --range |
        rgr sort -H -f 2 stdin |
        rgr prop cds.yml stdin -H -f 2 --prefix
    '

cat rgr.md.tmp

```

### R5 4600U Windows 11

| Command    |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-----------|------------:|---------:|---------:|-------------:|
| `tsv-sort` |  29.8 ± 1.2 |     27.1 |     33.3 |         1.00 |
| `rgr sort` |  49.3 ± 2.0 |     45.2 |     53.2 |  1.65 ± 0.09 |
| `rgr prop` | 408.3 ± 3.9 |    403.4 |    415.4 | 13.71 ± 0.56 |

