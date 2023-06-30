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

### i5-12500H Windows 11 WSL

| Command              |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|--------------:|--------:|--------:|------------:|
| `drop; gen;`         | 1.265 ± 0.025 |   1.228 |   1.304 |        1.00 |
| `d-g; range;`        | 3.061 ± 0.105 |   2.929 |   3.215 | 2.42 ± 0.10 |
| `clear; range;`      | 2.931 ± 0.140 |   2.735 |   3.155 | 2.32 ± 0.12 |
| `d-g; feature;`      | 3.127 ± 0.106 |   3.004 |   3.400 | 2.47 ± 0.10 |
| `d-g; feature; fsw;` | 9.358 ± 0.218 |   9.013 |   9.781 | 7.40 ± 0.23 |
| `d-g; sliding;`      | 6.060 ± 0.107 |   5.915 |   6.258 | 4.79 ± 0.13 |

### R5 4600U Windows 11 WSL

| Command              |       Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------|---------------:|--------:|--------:|------------:|
| `drop; gen;`         |  1.657 ± 0.030 |   1.610 |   1.698 |        1.00 |
| `d-g; range;`        |  5.785 ± 0.191 |   5.569 |   6.221 | 3.49 ± 0.13 |
| `clear; range;`      |  6.050 ± 0.123 |   5.907 |   6.281 | 3.65 ± 0.10 |
| `d-g; feature;`      |  5.694 ± 0.147 |   5.387 |   5.884 | 3.44 ± 0.11 |
| `d-g; feature; fsw;` | 15.643 ± 0.358 |  15.284 |  16.413 | 9.44 ± 0.28 |
| `d-g; sliding;`      |  7.880 ± 0.078 |   7.767 |   8.003 | 4.76 ± 0.10 |

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

### i5-12500H Windows 11 WSL

* feature

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 12.959 ± 1.023 |  11.919 |  15.276 | 1.66 ± 0.13 |
| `parallel -j 1` | 12.960 ± 0.419 |  12.506 |  13.734 | 1.66 ± 0.05 |
| `parallel -j 2` |  7.997 ± 0.096 |   7.845 |   8.148 | 1.03 ± 0.01 |
| `parallel -j 4` |  7.793 ± 0.025 |   7.754 |   7.830 |        1.00 |

* fsw


### R5 4600U Windows 11 WSL

* feature

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 27.161 ± 0.682 |  26.077 |  28.379 | 1.53 ± 0.05 |
| `parallel -j 1` | 28.967 ± 0.876 |  28.137 |  30.887 | 1.63 ± 0.06 |
| `parallel -j 2` | 18.355 ± 0.323 |  17.776 |  18.851 | 1.03 ± 0.03 |
| `parallel -j 4` | 17.807 ± 0.417 |  17.347 |  18.669 |        1.00 |

* fsw

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 41.269 ± 0.478 |  40.597 |  42.305 | 2.33 ± 0.07 |
| `parallel -j 1` | 50.651 ± 0.362 |  50.121 |  51.418 | 2.86 ± 0.08 |
| `parallel -j 2` | 26.043 ± 0.635 |  25.187 |  27.485 | 1.47 ± 0.05 |
| `parallel -j 4` | 17.721 ± 0.483 |  17.071 |  18.772 |        1.00 |

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
cd ~/gars
rm dump.rdb
redis-server

# gars
cd ~/gars
gars env

gars status drop

gars gen genome.fa.gz --piece 500000

hyperfine --warmup 1 --export-markdown locate.md.tmp \
    -n 'idx' \
    'gars locate -f T-DNA.CSHL.rg' \
    -n 'lapper' \
    'gars locate --lapper -f T-DNA.CSHL.rg' \
    -n 'zrange' \
    'gars locate --zrange -f T-DNA.CSHL.rg'

cat locate.md.tmp

```

### R7 5800 Windows 11 WSL

| Command  |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|-------------:|---------:|---------:|-------------:|
| `idx`    |   22.1 ± 0.5 |     21.3 |     25.8 |         1.00 |
| `lapper` | 930.5 ± 14.3 |    907.7 |    957.7 | 42.11 ± 1.23 |
| `zrange` | 2041.1 ± 9.6 |   2029.8 |   2056.4 | 92.37 ± 2.33 |

### i5-12500H Windows 11 WSL

| Command  |      Mean [ms] | Min [ms] | Max [ms] |      Relative |
|:---------|---------------:|---------:|---------:|--------------:|
| `idx`    |     21.4 ± 0.7 |     20.3 |     25.0 |          1.00 |
| `lapper` |  1037.6 ± 40.9 |    986.4 |   1105.4 |  48.38 ± 2.42 |
| `zrange` | 2330.1 ± 104.8 |   2254.5 |   2610.2 | 108.65 ± 5.92 |

### R5 4600U Windows 11 WSL

| Command  |     Mean [ms] | Min [ms] | Max [ms] |      Relative |
|:---------|--------------:|---------:|---------:|--------------:|
| `idx`    |    32.4 ± 1.4 |     30.8 |     37.6 |          1.00 |
| `lapper` | 1931.3 ± 64.0 |   1825.9 |   2012.9 |  59.64 ± 3.25 |
| `zrange` | 3852.4 ± 65.0 |   3780.5 |   3943.9 | 118.97 ± 5.53 |

### i7 8700K macOS Big Sur

| Command  |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|--------------:|---------:|---------:|-------------:|
| `idx`    |    62.2 ± 1.1 |     60.0 |     65.2 |         1.00 |
| `lapper` |   959.6 ± 9.4 |    945.4 |    974.2 | 15.43 ± 0.31 |
| `zrange` | 3161.5 ± 64.4 |   3097.3 |   3289.5 | 50.82 ± 1.36 |

## `rgr sort`

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

gars range T-DNA.CSHL.rg

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

### i5-12500H Windows 11 WSL

| Command            |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-------------------|--------------:|---------:|---------:|-------------:|
| `ctg - tsv-sort`   |  118.6 ± 11.1 |    105.4 |    146.5 |         1.00 |
| `ctg - rgr sort`   |  129.6 ± 12.8 |    116.1 |    154.8 |  1.09 ± 0.15 |
| `range - tsv-sort` | 1080.5 ± 17.0 |   1056.9 |   1116.2 |  9.11 ± 0.86 |
| `range - rgr sort` | 1991.1 ± 35.4 |   1959.2 |   2074.7 | 16.79 ± 1.59 |

### R5 4600U Windows 11 WSL

| Command           |      Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:------------------|---------------:|---------:|---------:|-------------:|
| `ctg: tsv-sort`   |    209.4 ± 5.4 |    198.0 |    220.4 |         1.00 |
| `ctg: rgr sort`   |   225.5 ± 10.5 |    209.0 |    238.1 |  1.08 ± 0.06 |
| `range: tsv-sort` |  2065.8 ± 87.3 |   1976.6 |   2190.4 |  9.87 ± 0.49 |
| `range: rgr sort` | 4066.6 ± 101.1 |   3905.2 |   4278.3 | 19.42 ± 0.70 |

## `rgr prop` and `gars anno`

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

gars range T-DNA.CSHL.rg

hyperfine --warmup 1 --export-markdown prop.md.tmp \
    -n 'ctg - cds - rgr prop' \
    '
    gars tsv -s "ctg:*" --range |
        rgr prop cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - cds - gars anno' \
    '
    gars tsv -s "ctg:*" --range |
        gars anno cds.yml stdin -H
    ' \
    -n 'ctg - repeats - rgr prop' \
    '
    gars tsv -s "ctg:*" --range |
        rgr prop repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - repeats - gars anno' \
    '
    gars tsv -s "ctg:*" --range |
        gars anno repeats.yml stdin -H
    ' \
    -n 'range - cds - rgr prop' \
    '
    gars tsv -s "range:*" --range |
        rgr prop cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - cds - gars anno' \
    '
    gars tsv -s "range:*" --range |
        gars anno cds.yml stdin -H
    ' \
    -n 'range - repeats - rgr prop' \
    '
    gars tsv -s "range:*" --range |
        rgr prop repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - repeats - gars anno' \
    '
    gars tsv -s "range:*" --range |
        gars anno repeats.yml stdin -H
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

### i5-12500H Windows 11 WSL

| Command                       |      Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:------------------------------|---------------:|---------:|---------:|-------------:|
| `ctg - cds - rgr prop`        |   445.1 ± 13.0 |    429.6 |    466.4 |         1.00 |
| `ctg - cds - gars anno`       |   495.4 ± 11.7 |    481.8 |    520.2 |  1.11 ± 0.04 |
| `ctg - repeats - rgr prop`    |   963.4 ± 14.8 |    939.6 |    980.6 |  2.16 ± 0.07 |
| `ctg - repeats - gars anno`   |  1017.9 ± 18.5 |    995.6 |   1053.4 |  2.29 ± 0.08 |
| `range - cds - rgr prop`      | 5029.5 ± 147.1 |   4751.8 |   5208.8 | 11.30 ± 0.47 |
| `range - cds - gars anno`     |  2141.9 ± 46.9 |   2090.6 |   2237.5 |  4.81 ± 0.18 |
| `range - repeats - rgr prop`  | 7795.8 ± 240.8 |   7454.3 |   8113.4 | 17.51 ± 0.75 |
| `range - repeats - gars anno` | 2709.9 ± 130.4 |   2602.4 |   3047.7 |  6.09 ± 0.34 |

### R5 4600U Windows 11 WSL

| Command                       |       Mean [ms] | Min [ms] | Max [ms] |      Relative |
|:------------------------------|----------------:|---------:|---------:|--------------:|
| `ctg - cds - rgr prop`        |     542.5 ± 5.7 |    534.3 |    550.3 |          1.00 |
| `ctg - cds - gars anno`       | 25040.8 ± 106.2 |  24871.5 |  25164.0 |  46.15 ± 0.53 |
| `ctg - repeats - rgr prop`    |    952.5 ± 40.3 |    928.0 |   1065.2 |   1.76 ± 0.08 |
| `ctg - repeats - gars anno`   | 57937.1 ± 276.4 |  57572.3 |  58375.0 | 106.79 ± 1.24 |
| `range - cds - rgr prop`      |   7460.0 ± 39.3 |   7394.5 |   7524.7 |  13.75 ± 0.16 |
| `range - cds - gars anno`     | 24846.5 ± 162.7 |  24603.7 |  25083.0 |  45.80 ± 0.57 |
| `range - repeats - rgr prop`  |  11292.8 ± 73.2 |  11206.9 |  11464.7 |  20.81 ± 0.26 |
| `range - repeats - gars anno` | 54754.9 ± 584.1 |  54087.1 |  55794.9 | 100.92 ± 1.52 |
