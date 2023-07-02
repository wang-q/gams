# Atha

## `gars`

```shell
# redis
cd ~/data/gars/Atha/
rm dump.rdb
redis-server

# gars
cd ~/data/gars/Atha/
gars env

hyperfine --warmup 1 --export-markdown gars.md.tmp \
    -n 'drop; gen;' \
    '
    gars status drop; gars gen genome/genome.fa.gz --piece 500000;
    ' \
    -n 'd-g; range;' \
    '
    gars status drop; gars gen genome/genome.fa.gz --piece 500000;
    gars range features/T-DNA.CSHL.rg;
    ' \
    -n 'clear; range;' \
    '
    gars clear range;
    gars range features/T-DNA.CSHL.rg;
    ' \
    -n 'd-g; feature;' \
    '
    gars status drop; gars gen genome/genome.fa.gz --piece 500000;
    gars feature features/T-DNA.CSHL.rg --tag CSHL;
    ' \
    -n 'd-g; feature; fsw;' \
    '
    gars status drop; gars gen genome/genome.fa.gz --piece 500000;
    gars feature features/T-DNA.CSHL.rg --tag CSHL;
    gars fsw;
    ' \
    -n 'd-g; sliding;' \
    '
    gars status drop; gars gen genome/genome.fa.gz --piece 500000;
    gars sliding --size 100 --step 20 --lag 50 > /dev/null;
    '

cat gars.md.tmp

```

### R7 5800 Windows 11 WSL

| Command              |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|--------------:|--------:|--------:|------------:|
| `drop; gen;`         | 1.095 ± 0.009 |   1.083 |   1.111 |        1.00 |
| `d-g; range;`        | 2.929 ± 0.101 |   2.774 |   3.103 | 2.67 ± 0.09 |
| `clear; range;`      | 2.652 ± 0.123 |   2.446 |   2.858 | 2.42 ± 0.11 |
| `d-g; feature;`      | 2.868 ± 0.058 |   2.777 |   2.954 | 2.62 ± 0.06 |
| `d-g; feature; fsw;` | 9.108 ± 0.153 |   9.015 |   9.519 | 8.32 ± 0.16 |
| `d-g; sliding;`      | 5.993 ± 0.021 |   5.965 |   6.024 | 5.47 ± 0.05 |

### E5-2680 v3 RHEL 7.7

| Command              |       Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|---------------:|--------:|--------:|------------:|
| `drop; gen;`         |  1.979 ± 0.017 |   1.949 |   2.011 | 1.30 ± 0.02 |
| `d-g; range;`        |  3.086 ± 0.048 |   3.012 |   3.153 | 2.02 ± 0.05 |
| `clear; range;`      |  1.527 ± 0.026 |   1.488 |   1.578 |        1.00 |
| `d-g; feature;`      |  3.129 ± 0.033 |   3.089 |   3.206 | 2.05 ± 0.04 |
| `d-g; feature; fsw;` | 13.935 ± 0.175 |  13.608 |  14.149 | 9.13 ± 0.19 |
| `d-g; sliding;`      | 10.842 ± 0.290 |  10.398 |  11.129 | 7.10 ± 0.23 |

### Apple M2 macOS 13.4

| Command              |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|--------------:|--------:|--------:|------------:|
| `drop; gen;`         | 1.256 ± 0.006 |   1.248 |   1.266 |        1.00 |
| `d-g; range;`        | 2.334 ± 0.010 |   2.322 |   2.350 | 1.86 ± 0.01 |
| `clear; range;`      | 1.616 ± 0.007 |   1.609 |   1.633 | 1.29 ± 0.01 |
| `d-g; feature;`      | 2.376 ± 0.020 |   2.343 |   2.409 | 1.89 ± 0.02 |
| `d-g; feature; fsw;` | 7.840 ± 0.037 |   7.811 |   7.941 | 6.24 ± 0.04 |
| `d-g; sliding;`      | 4.872 ± 0.014 |   4.854 |   4.895 | 3.88 ± 0.02 |

## threads

```shell
# redis
cd ~/data/gars/Atha/
rm dump.rdb
redis-server

# gars
cd ~/data/gars/Atha/
gars env

# feature
gars status drop; gars gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars clear feature;
    gars feature features/T-DNA.CSHL.rg --tag CSHL;
    gars feature features/T-DNA.FLAG.rg --tag FLAG;
    gars feature features/T-DNA.MX.rg   --tag MX;
    gars feature features/T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gars clear feature;
    parallel -j 1 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gars clear feature;
    parallel -j 2 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gars clear feature;
    parallel -j 4 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

# fsw
gars status drop; gars gen genome/genome.fa.gz --piece 500000;

gars feature features/T-DNA.CSHL.rg --tag CSHL;
gars feature features/T-DNA.FLAG.rg --tag FLAG;
gars feature features/T-DNA.MX.rg   --tag MX;
gars feature features/T-DNA.RATM.rg --tag RATM;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars fsw > /dev/null;
    ' \
    -n 'parallel -j 1' \
    '
    cat genome/chr.sizes |
        cut -f 1 |
        parallel -j 1 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    ' \
    -n 'parallel -j 2' \
    '
    cat genome/chr.sizes |
        cut -f 1 |
        parallel -j 2 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    ' \
    -n 'parallel -j 4' \
    '
    cat genome/chr.sizes |
        cut -f 1 |
        parallel -j 4 '\''
            gars fsw --ctg "ctg:{}:*"
            '\'' \
        > /dev/null
    '

cat threads.md.tmp

```

### R7 5800 Windows 11 WSL

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

### E5-2680 v3 RHEL 7.7

* feature

| Command         |      Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|--------------:|--------:|--------:|------------:|
| `serial`        | 7.308 ± 0.058 |   7.247 |   7.436 | 1.49 ± 0.03 |
| `parallel -j 1` | 7.749 ± 0.043 |   7.691 |   7.841 | 1.58 ± 0.03 |
| `parallel -j 2` | 5.191 ± 0.117 |   5.026 |   5.404 | 1.06 ± 0.03 |
| `parallel -j 4` | 4.918 ± 0.090 |   4.830 |   5.042 |        1.00 |

* fsw

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 44.774 ± 1.161 |  42.892 |  46.623 | 2.75 ± 0.08 |
| `parallel -j 1` | 48.743 ± 0.765 |  47.453 |  49.555 | 3.00 ± 0.06 |
| `parallel -j 2` | 25.205 ± 0.558 |  24.065 |  25.980 | 1.55 ± 0.04 |
| `parallel -j 4` | 16.273 ± 0.178 |  15.946 |  16.552 |        1.00 |

### Apple M2 macOS 13.4

* feature

| Command         |      Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|--------------:|--------:|--------:|------------:|
| `serial`        | 7.562 ± 0.032 |   7.500 |   7.617 | 1.47 ± 0.01 |
| `parallel -j 1` | 8.013 ± 0.021 |   7.979 |   8.045 | 1.56 ± 0.01 |
| `parallel -j 2` | 6.302 ± 0.019 |   6.263 |   6.321 | 1.22 ± 0.01 |
| `parallel -j 4` | 5.146 ± 0.019 |   5.121 |   5.176 |        1.00 |

* fsw

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 21.995 ± 0.026 |  21.957 |  22.044 | 2.34 ± 0.01 |
| `parallel -j 1` | 25.078 ± 0.026 |  25.038 |  25.122 | 2.66 ± 0.01 |
| `parallel -j 2` | 14.065 ± 0.021 |  14.034 |  14.098 | 1.49 ± 0.00 |
| `parallel -j 4` |  9.413 ± 0.024 |   9.369 |   9.464 |        1.00 |

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

gars status drop; gars gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars clear feature;
    gars feature features/T-DNA.CSHL.rg --tag CSHL;
    gars feature features/T-DNA.FLAG.rg --tag FLAG;
    gars feature features/T-DNA.MX.rg   --tag MX;
    gars feature features/T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gars clear feature;
    parallel -j 1 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gars clear feature;
    parallel -j 2 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gars clear feature;
    parallel -j 4 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

```

### R7 5800 Windows 11 WSL

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
cd ~/data/gars/Atha/
rm dump.rdb
redis-server

# gars
cd ~/data/gars/Atha/
gars env

gars status drop

gars gen genome/genome.fa.gz --piece 500000

hyperfine --warmup 1 --export-markdown locate.md.tmp \
    -n 'idx' \
    'gars locate -f features/T-DNA.CSHL.rg' \
    -n 'lapper' \
    'gars locate --lapper -f features/T-DNA.CSHL.rg' \
    -n 'zrange' \
    'gars locate --zrange -f features/T-DNA.CSHL.rg'

cat locate.md.tmp

```

### R7 5800 Windows 11 WSL

| Command  |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|-------------:|---------:|---------:|-------------:|
| `idx`    |   22.1 ± 0.5 |     21.3 |     25.8 |         1.00 |
| `lapper` | 930.5 ± 14.3 |    907.7 |    957.7 | 42.11 ± 1.23 |
| `zrange` | 2041.1 ± 9.6 |   2029.8 |   2056.4 | 92.37 ± 2.33 |

### E5-2680 v3 RHEL 7.7

| Command  |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|--------------:|---------:|---------:|-------------:|
| `idx`    |    48.0 ± 2.4 |     45.0 |     53.9 |         1.00 |
| `lapper` |  613.3 ± 21.8 |    583.1 |    658.5 | 12.78 ± 0.79 |
| `zrange` | 2377.9 ± 27.8 |   2336.1 |   2429.0 | 49.54 ± 2.59 |

### Apple M2 macOS 13.4

| Command  |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|-------------:|---------:|---------:|-------------:|
| `idx`    |   25.6 ± 0.5 |     24.6 |     29.8 |         1.00 |
| `lapper` |  560.5 ± 2.1 |    557.0 |    563.6 | 21.85 ± 0.45 |
| `zrange` | 2030.1 ± 7.9 |   2021.8 |   2049.7 | 79.15 ± 1.63 |

## `rgr sort`

```shell
# redis
cd ~/data/gars/Atha/
rm dump.rdb
redis-server

# gars
cd ~/data/gars/Atha/
gars env

gars status drop

gars gen genome/genome.fa.gz --piece 500000

gars range features/T-DNA.CSHL.rg

hyperfine --warmup 1 --export-markdown rgr.md.tmp \
    -n 'ctg - tsv-sort' \
    '
    gars tsv -s "ctg:*" |
        keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n
    ' \
    -n 'ctg - rgr sort' \
    '
    gars tsv -s "ctg:*" --range |
        rgr sort -H -f 2 stdin
    ' \
    -n 'range - tsv-sort' \
    '
    gars tsv -s "range:*" |
        keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n
    ' \
    -n 'range - rgr sort' \
    '
    gars tsv -s "range:*" --range |
        rgr sort -H -f 2 stdin
    '

cat rgr.md.tmp

```

### R7 5800 Windows 11 WSL

| Command            |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-------------------|--------------:|---------:|---------:|-------------:|
| `ctg - tsv-sort`   |    92.1 ± 5.5 |     84.0 |    104.1 |         1.00 |
| `ctg - rgr sort`   |   101.4 ± 8.3 |     92.0 |    123.9 |  1.10 ± 0.11 |
| `range - tsv-sort` |  996.9 ± 60.8 |    919.6 |   1096.1 | 10.82 ± 0.92 |
| `range - rgr sort` | 1740.1 ± 50.5 |   1666.5 |   1828.8 | 18.88 ± 1.25 |

### Apple M2 macOS 13.4

| Command            |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-------------------|--------------:|---------:|---------:|-------------:|
| `ctg - tsv-sort`   |    65.4 ± 1.1 |     62.8 |     70.3 |         1.00 |
| `ctg - rgr sort`   |    69.1 ± 3.9 |     67.6 |     93.1 |  1.06 ± 0.06 |
| `range - tsv-sort` |   663.1 ± 3.3 |    656.4 |    667.3 | 10.13 ± 0.17 |
| `range - rgr sort` | 1147.0 ± 11.4 |   1122.8 |   1169.8 | 17.53 ± 0.34 |

## `rgr prop` and `gars anno`

```shell
# redis
cd ~/data/gars/Atha/
rm dump.rdb
redis-server

# gars
cd ~/data/gars/Atha/
gars env

gars status drop

gars gen genome/genome.fa.gz --piece 500000

gars range features/T-DNA.CSHL.rg

hyperfine --warmup 1 --export-markdown prop.md.tmp \
    -n 'ctg - cds - rgr prop' \
    '
    gars tsv -s "ctg:*" --range |
        rgr prop genome/cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - cds - gars anno' \
    '
    gars tsv -s "ctg:*" --range |
        gars anno genome/cds.yml stdin -H
    ' \
    -n 'ctg - repeats - rgr prop' \
    '
    gars tsv -s "ctg:*" --range |
        rgr prop genome/repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - repeats - gars anno' \
    '
    gars tsv -s "ctg:*" --range |
        gars anno genome/repeats.yml stdin -H
    ' \
    -n 'range - cds - rgr prop' \
    '
    gars tsv -s "range:*" --range |
        rgr prop genome/cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - cds - gars anno' \
    '
    gars tsv -s "range:*" --range |
        gars anno genome/cds.yml stdin -H
    ' \
    -n 'range - repeats - rgr prop' \
    '
    gars tsv -s "range:*" --range |
        rgr prop genome/repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - repeats - gars anno' \
    '
    gars tsv -s "range:*" --range |
        gars anno genome/repeats.yml stdin -H
    '

cat prop.md.tmp

```

### R7 5800 Windows 11 WSL

| Command                       |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:------------------------------|--------------:|---------:|---------:|-------------:|
| `ctg - cds - rgr prop`        |   388.5 ± 2.9 |    383.8 |    392.6 |         1.00 |
| `ctg - cds - gars anno`       |  418.0 ± 11.8 |    410.0 |    449.7 |  1.08 ± 0.03 |
| `ctg - repeats - rgr prop`    |   731.7 ± 9.4 |    720.0 |    748.1 |  1.88 ± 0.03 |
| `ctg - repeats - gars anno`   |   753.7 ± 3.0 |    748.8 |    759.4 |  1.94 ± 0.02 |
| `range - cds - rgr prop`      | 5686.0 ± 13.0 |   5669.5 |   5713.4 | 14.63 ± 0.11 |
| `range - cds - gars anno`     | 1760.8 ± 49.7 |   1705.4 |   1887.6 |  4.53 ± 0.13 |
| `range - repeats - rgr prop`  |  8776.2 ± 8.8 |   8765.1 |   8796.1 | 22.59 ± 0.17 |
| `range - repeats - gars anno` | 2088.1 ± 54.3 |   2032.1 |   2179.1 |  5.37 ± 0.15 |

### Apple M2 macOS 13.4

| Command                       |     Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------------------------|--------------:|---------:|---------:|------------:|
| `ctg - cds - rgr prop`        |   498.2 ± 5.7 |    486.9 |    504.7 |        1.00 |
| `ctg - cds - gars anno`       |   512.9 ± 2.9 |    510.1 |    517.5 | 1.03 ± 0.01 |
| `ctg - repeats - rgr prop`    |  1057.5 ± 3.2 |   1053.6 |   1064.4 | 2.12 ± 0.03 |
| `ctg - repeats - gars anno`   |  1082.4 ± 5.8 |   1077.1 |   1097.5 | 2.17 ± 0.03 |
| `range - cds - rgr prop`      | 3102.5 ± 12.2 |   3085.7 |   3118.3 | 6.23 ± 0.08 |
| `range - cds - gars anno`     |  1479.7 ± 5.7 |   1471.3 |   1489.5 | 2.97 ± 0.04 |
| `range - repeats - rgr prop`  | 4221.7 ± 18.5 |   4186.9 |   4252.5 | 8.47 ± 0.10 |
| `range - repeats - gars anno` |  2013.6 ± 9.3 |   2007.8 |   2039.6 | 4.04 ± 0.05 |
