# Atha

## `gams`

```shell
# Avoid NTFS
mkdir -p ~/gams
cd ~/gams

cp -R ~/data/gams/Atha/genome .
cp -R ~/data/gams/Atha/features .

# redis
rm dump.rdb
redis-server &

# gams
gams env

# flamegraph
#sudo apt install linux-tools-generic
#export PERF=/usr/lib/linux-tools/5.4.0-174-generic/perf
#flamegraph -- gams status drop
#flamegraph -- gams gen genome/genome.fa.gz --piece 500000

hyperfine --warmup 1 --export-markdown gams.md.tmp \
    -n 'drop; gen;' \
    '
    gams status drop; gams gen genome/genome.fa.gz --piece 500000;
    ' \
    -n 'd-g; range;' \
    '
    gams status drop; gams gen genome/genome.fa.gz --piece 500000;
    gams range features/T-DNA.CSHL.rg;
    ' \
    -n 'clear; range;' \
    '
    gams clear range feature;
    gams range features/T-DNA.CSHL.rg;
    ' \
    -n 'clear; feature;' \
    '
    gams clear range feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL;
    ' \
    -n 'clear; feature; fsw;' \
    '
    gams clear range feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL;
    gams fsw;
    ' \
    -n 'clear; sliding;' \
    '
    gams clear range feature;
    gams sliding --size 100 --step 20 --lag 50 > /dev/null;
    '

cat gams.md.tmp

```

### R7 5800 Windows 11 WSL

| Command                |      Mean [s] | Min [s] | Max [s] |    Relative |
|:-----------------------|--------------:|--------:|--------:|------------:|
| `drop; gen;`           | 1.095 ± 0.009 |   1.083 |   1.111 |        1.00 |
| `d-g; range;`          | 2.929 ± 0.101 |   2.774 |   3.103 | 2.67 ± 0.09 |
| `clear; range;`        | 2.652 ± 0.123 |   2.446 |   2.858 | 2.42 ± 0.11 |
| `clear; feature;`      | 2.868 ± 0.058 |   2.777 |   2.954 | 2.62 ± 0.06 |
| `clear; feature; fsw;` | 9.108 ± 0.153 |   9.015 |   9.519 | 8.32 ± 0.16 |
| `clear; sliding;`      | 5.993 ± 0.021 |   5.965 |   6.024 | 5.47 ± 0.05 |

### i5-12500H Windows 11 WSL

| Command                |      Mean [s] | Min [s] | Max [s] |     Relative |
|:-----------------------|--------------:|--------:|--------:|-------------:|
| `drop; gen;`           | 1.122 ± 0.029 |   1.083 |   1.173 |  8.20 ± 0.42 |
| `d-g; range;`          | 1.242 ± 0.016 |   1.223 |   1.270 |  9.07 ± 0.42 |
| `clear; range;`        | 0.137 ± 0.006 |   0.129 |   0.153 |         1.00 |
| `clear; feature;`      | 0.169 ± 0.006 |   0.161 |   0.180 |  1.24 ± 0.07 |
| `clear; feature; fsw;` | 4.851 ± 0.042 |   4.808 |   4.925 | 35.45 ± 1.61 |
| `clear; sliding;`      | 4.407 ± 0.051 |   4.340 |   4.490 | 32.21 ± 1.48 |

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

| Command                |      Mean [s] | Min [s] | Max [s] |     Relative |
|:-----------------------|--------------:|--------:|--------:|-------------:|
| `drop; gen;`           | 4.074 ± 0.018 |   4.051 |   4.104 | 37.25 ± 0.64 |
| `d-g; range;`          | 4.229 ± 0.008 |   4.214 |   4.240 | 38.67 ± 0.64 |
| `clear; range;`        | 0.109 ± 0.002 |   0.107 |   0.116 |         1.00 |
| `clear; feature;`      | 0.172 ± 0.002 |   0.170 |   0.179 |  1.57 ± 0.03 |
| `clear; feature; fsw;` | 4.278 ± 0.006 |   4.264 |   4.286 | 39.13 ± 0.65 |
| `clear; sliding;`      | 4.053 ± 0.017 |   4.038 |   4.078 | 37.07 ± 0.63 |

## Batch size

```shell
cd ~/gams/

# redis
rm dump.rdb
redis-server &

# gams
gams env

# feature
gams status drop; gams gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown batch.md.tmp \
    -n 'size 1' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL --size 1;
    ' \
    -n 'size 10' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL --size 10;
    ' \
    -n 'size 50' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL --size 50;
    ' \
    -n 'size 100' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL --size 100;
    ' \
    -n 'size 1000' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL --size 1000;
    '

cat batch.md.tmp

```

### i5-12500H Windows 11 WSL

| Command     |      Mean [s] | Min [s] | Max [s] |    Relative |
|:------------|--------------:|--------:|--------:|------------:|
| `size 1`    | 1.057 ± 0.044 |   1.008 |   1.131 | 6.32 ± 0.33 |
| `size 10`   | 0.290 ± 0.017 |   0.255 |   0.313 | 1.73 ± 0.11 |
| `size 50`   | 0.173 ± 0.007 |   0.165 |   0.189 | 1.03 ± 0.06 |
| `size 100`  | 0.167 ± 0.005 |   0.159 |   0.180 |        1.00 |
| `size 1000` | 0.169 ± 0.010 |   0.154 |   0.189 | 1.01 ± 0.07 |

### Apple M2 macOS 13.4

| Command     |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------|------------:|---------:|---------:|------------:|
| `size 1`    | 612.1 ± 5.7 |    605.2 |    623.7 | 3.60 ± 0.07 |
| `size 10`   | 237.8 ± 2.9 |    234.3 |    243.0 | 1.40 ± 0.03 |
| `size 50`   | 177.8 ± 2.1 |    175.3 |    183.9 | 1.05 ± 0.02 |
| `size 100`  | 169.9 ± 2.7 |    167.2 |    178.0 |        1.00 |
| `size 1000` | 174.0 ± 5.5 |    162.0 |    180.7 | 1.02 ± 0.04 |

## threads

```shell
cd ~/gams/

# redis
rm dump.rdb
redis-server &

# gams
gams env

# feature
gams status drop; gams gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gams clear feature;
    gams feature features/T-DNA.CSHL.rg --tag CSHL;
    gams feature features/T-DNA.FLAG.rg --tag FLAG;
    gams feature features/T-DNA.MX.rg   --tag MX;
    gams feature features/T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gams clear feature;
    parallel -j 1 "
        echo {}
        gams feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gams clear feature;
    parallel -j 2 "
        echo {}
        gams feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gams clear feature;
    parallel -j 4 "
        echo {}
        gams feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

# fsw
gams status drop
gams gen genome/genome.fa.gz --piece 500000

gams feature features/T-DNA.CSHL.rg --tag CSHL;
gams feature features/T-DNA.FLAG.rg --tag FLAG;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'parallel 1' \
    '
    gams fsw --parallel 1 > /dev/null;
    ' \
    -n 'parallel 2' \
    '
    gams fsw --parallel 2 > /dev/null;
    ' \
    -n 'parallel 4' \
    '
    gams fsw --parallel 4 > /dev/null;
    ' \
    -n 'parallel 8' \
    '
    gams fsw --parallel 8 > /dev/null;
    ' \
    -n 'parallel 12' \
    '
    gams fsw --parallel 12 > /dev/null;
    ' \
    -n 'parallel 16' \
    '
    gams fsw --parallel 16 > /dev/null;
    '

cat threads.md.tmp

# sliding
gams status drop; gams gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'parallel 1' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 1 \
        > /dev/null
    ' \
    -n 'parallel 2' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 2 \
        > /dev/null
    ' \
    -n 'parallel 4' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 4 \
        > /dev/null
    ' \
    -n 'parallel 8' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 8 \
        > /dev/null
    ' \
    -n 'parallel 12' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 12 \
        > /dev/null
    ' \
    -n 'parallel 16' \
    '
    gams sliding \
        --ctg "ctg:1:*" \
        --size 100 --step 5 \
        --lag 200 --threshold 3.0 --influence 1.0 \
        --parallel 16 \
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

| Command         |     Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------------|--------------:|---------:|---------:|------------:|
| `serial`        |  683.1 ± 48.6 |    629.4 |    776.0 |        1.00 |
| `parallel -j 1` | 1020.4 ± 46.8 |    944.6 |   1078.5 | 1.49 ± 0.13 |
| `parallel -j 2` |  825.3 ± 66.4 |    739.3 |    919.9 | 1.21 ± 0.13 |
| `parallel -j 4` |  873.8 ± 82.9 |    775.0 |    998.5 | 1.28 ± 0.15 |

* fsw

| Command       |       Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------|---------------:|--------:|--------:|------------:|
| `parallel 1`  | 11.384 ± 0.129 |  11.269 |  11.660 | 5.60 ± 0.10 |
| `parallel 2`  |  6.502 ± 0.101 |   6.390 |   6.753 | 3.20 ± 0.07 |
| `parallel 4`  |  3.928 ± 0.062 |   3.872 |   4.090 | 1.93 ± 0.04 |
| `parallel 8`  |  2.657 ± 0.049 |   2.594 |   2.746 | 1.31 ± 0.03 |
| `parallel 12` |  2.237 ± 0.016 |   2.203 |   2.254 | 1.10 ± 0.02 |
| `parallel 16` |  2.034 ± 0.029 |   1.991 |   2.072 |        1.00 |

* sliding

| Command       |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------|--------------:|--------:|--------:|------------:|
| `parallel 1`  | 6.209 ± 0.087 |   6.115 |   6.345 | 4.71 ± 0.15 |
| `parallel 2`  | 3.410 ± 0.030 |   3.374 |   3.472 | 2.59 ± 0.08 |
| `parallel 4`  | 2.013 ± 0.022 |   1.979 |   2.044 | 1.53 ± 0.05 |
| `parallel 8`  | 1.470 ± 0.037 |   1.421 |   1.544 | 1.11 ± 0.04 |
| `parallel 12` | 1.319 ± 0.037 |   1.285 |   1.400 |        1.00 |
| `parallel 16` | 1.346 ± 0.050 |   1.294 |   1.415 | 1.02 ± 0.05 |

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

* sliding

| Command       |       Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------|---------------:|--------:|--------:|------------:|
| `parallel 1`  | 11.400 ± 0.229 |  11.097 |  11.932 | 9.70 ± 0.37 |
| `parallel 2`  |  6.368 ± 0.101 |   6.135 |   6.467 | 5.42 ± 0.19 |
| `parallel 4`  |  3.375 ± 0.023 |   3.338 |   3.407 | 2.87 ± 0.09 |
| `parallel 8`  |  1.837 ± 0.029 |   1.821 |   1.917 | 1.56 ± 0.06 |
| `parallel 12` |  1.324 ± 0.007 |   1.314 |   1.335 | 1.13 ± 0.04 |
| `parallel 16` |  1.175 ± 0.038 |   1.108 |   1.246 |        1.00 |

### Apple M2 macOS 13.4

* feature

| Command         |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:----------------|------------:|---------:|---------:|------------:|
| `serial`        | 794.6 ± 5.7 |    788.1 |    806.5 | 1.31 ± 0.02 |
| `parallel -j 1` | 945.2 ± 9.4 |    934.0 |    960.7 | 1.56 ± 0.02 |
| `parallel -j 2` | 675.3 ± 4.3 |    667.1 |    683.2 | 1.12 ± 0.01 |
| `parallel -j 4` | 605.0 ± 6.5 |    597.9 |    616.0 |        1.00 |

* fsw

| Command       |       Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------|---------------:|--------:|--------:|------------:|
| `parallel 1`  | 10.920 ± 0.272 |  10.807 |  11.691 | 4.63 ± 0.13 |
| `parallel 2`  |  5.582 ± 0.014 |   5.564 |   5.608 | 2.37 ± 0.03 |
| `parallel 4`  |  2.960 ± 0.015 |   2.931 |   2.982 | 1.26 ± 0.02 |
| `parallel 8`  |  2.396 ± 0.021 |   2.363 |   2.424 | 1.02 ± 0.02 |
| `parallel 12` |  2.358 ± 0.032 |   2.320 |   2.428 |        1.00 |
| `parallel 16` |  2.366 ± 0.032 |   2.321 |   2.421 | 1.00 ± 0.02 |

* sliding

| Command       |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------|--------------:|--------:|--------:|------------:|
| `parallel 1`  | 5.722 ± 0.011 |   5.709 |   5.740 | 5.32 ± 0.06 |
| `parallel 2`  | 2.952 ± 0.024 |   2.940 |   3.020 | 2.74 ± 0.04 |
| `parallel 4`  | 1.578 ± 0.023 |   1.557 |   1.625 | 1.47 ± 0.03 |
| `parallel 8`  | 1.112 ± 0.014 |   1.093 |   1.136 | 1.03 ± 0.02 |
| `parallel 12` | 1.089 ± 0.013 |   1.072 |   1.109 | 1.01 ± 0.02 |
| `parallel 16` | 1.076 ± 0.012 |   1.063 |   1.092 |        1.00 |

## `gams locate`

```shell
cd ~/gams/

# redis
rm dump.rdb
redis-server &

# gams
gams env

gams status drop

gams gen genome/genome.fa.gz --piece 500000

hyperfine --warmup 1 --export-markdown locate.md.tmp \
    -n 'idx' \
    'gams locate -f features/T-DNA.CSHL.rg' \
    -n 'lapper' \
    'gams locate --lapper -f features/T-DNA.CSHL.rg' \
    -n 'zrange' \
    'gams locate --zrange -f features/T-DNA.CSHL.rg'

cat locate.md.tmp

```

### R7 5800 Windows 11 WSL

| Command  |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|-------------:|---------:|---------:|-------------:|
| `idx`    |   22.1 ± 0.5 |     21.3 |     25.8 |         1.00 |
| `lapper` | 930.5 ± 14.3 |    907.7 |    957.7 | 42.11 ± 1.23 |
| `zrange` | 2041.1 ± 9.6 |   2029.8 |   2056.4 | 92.37 ± 2.33 |

### i5-12500H Windows 11 WSL

| Command  |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:---------|--------------:|---------:|---------:|-------------:|
| `idx`    |    20.6 ± 1.2 |     18.7 |     29.0 |         1.00 |
| `lapper` |  909.5 ± 18.6 |    882.1 |    934.5 | 44.24 ± 2.84 |
| `zrange` | 1456.8 ± 29.4 |   1428.8 |   1506.3 | 70.87 ± 4.54 |

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
cd ~/data/gams/Atha/
rm dump.rdb
redis-server

# gams
cd ~/data/gams/Atha/
gams env

gams status drop

gams gen genome/genome.fa.gz --piece 500000

gams range features/T-DNA.CSHL.rg

hyperfine --warmup 1 --export-markdown rgr.md.tmp \
    -n 'ctg - tsv-sort' \
    '
    gams tsv -s "ctg:*" |
        keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n
    ' \
    -n 'ctg - rgr sort' \
    '
    gams tsv -s "ctg:*" --range |
        rgr sort -H -f 2 stdin
    ' \
    -n 'range - tsv-sort' \
    '
    gams tsv -s "range:*" |
        keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n
    ' \
    -n 'range - rgr sort' \
    '
    gams tsv -s "range:*" --range |
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

### E5-2680 v3 RHEL 7.7

| Command            |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-------------------|--------------:|---------:|---------:|-------------:|
| `ctg - tsv-sort`   |    61.5 ± 1.6 |     58.7 |     65.3 |         1.00 |
| `ctg - rgr sort`   |    66.0 ± 1.9 |     61.8 |     69.4 |  1.07 ± 0.04 |
| `range - tsv-sort` | 1167.9 ± 19.4 |   1137.2 |   1202.9 | 18.98 ± 0.59 |
| `range - rgr sort` | 1084.7 ± 21.8 |   1049.3 |   1112.3 | 17.63 ± 0.59 |

### Apple M2 macOS 13.4

| Command            |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:-------------------|--------------:|---------:|---------:|-------------:|
| `ctg - tsv-sort`   |    65.4 ± 1.1 |     62.8 |     70.3 |         1.00 |
| `ctg - rgr sort`   |    69.1 ± 3.9 |     67.6 |     93.1 |  1.06 ± 0.06 |
| `range - tsv-sort` |   663.1 ± 3.3 |    656.4 |    667.3 | 10.13 ± 0.17 |
| `range - rgr sort` | 1147.0 ± 11.4 |   1122.8 |   1169.8 | 17.53 ± 0.34 |

## `rgr prop` and `gams anno`

```shell
# redis
cd ~/data/gams/Atha/
rm dump.rdb
redis-server

# gams
cd ~/data/gams/Atha/
gams env

gams status drop

gams gen genome/genome.fa.gz --piece 500000

gams range features/T-DNA.CSHL.rg

hyperfine --warmup 1 --export-markdown prop.md.tmp \
    -n 'ctg - cds - rgr prop' \
    '
    gams tsv -s "ctg:*" --range |
        rgr prop genome/cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - cds - gams anno' \
    '
    gams tsv -s "ctg:*" --range |
        gams anno genome/cds.yml stdin -H
    ' \
    -n 'ctg - repeats - rgr prop' \
    '
    gams tsv -s "ctg:*" --range |
        rgr prop genome/repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'ctg - repeats - gams anno' \
    '
    gams tsv -s "ctg:*" --range |
        gams anno genome/repeats.yml stdin -H
    ' \
    -n 'range - cds - rgr prop' \
    '
    gams tsv -s "range:*" --range |
        rgr prop genome/cds.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - cds - gams anno' \
    '
    gams tsv -s "range:*" --range |
        gams anno genome/cds.yml stdin -H
    ' \
    -n 'range - repeats - rgr prop' \
    '
    gams tsv -s "range:*" --range |
        rgr prop genome/repeats.yml stdin -H -f 2 --prefix
    ' \
    -n 'range - repeats - gams anno' \
    '
    gams tsv -s "range:*" --range |
        gams anno genome/repeats.yml stdin -H
    '

cat prop.md.tmp

```

### R7 5800 Windows 11 WSL

| Command                       |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:------------------------------|--------------:|---------:|---------:|-------------:|
| `ctg - cds - rgr prop`        |   388.5 ± 2.9 |    383.8 |    392.6 |         1.00 |
| `ctg - cds - gams anno`       |  418.0 ± 11.8 |    410.0 |    449.7 |  1.08 ± 0.03 |
| `ctg - repeats - rgr prop`    |   731.7 ± 9.4 |    720.0 |    748.1 |  1.88 ± 0.03 |
| `ctg - repeats - gams anno`   |   753.7 ± 3.0 |    748.8 |    759.4 |  1.94 ± 0.02 |
| `range - cds - rgr prop`      | 5686.0 ± 13.0 |   5669.5 |   5713.4 | 14.63 ± 0.11 |
| `range - cds - gams anno`     | 1760.8 ± 49.7 |   1705.4 |   1887.6 |  4.53 ± 0.13 |
| `range - repeats - rgr prop`  |  8776.2 ± 8.8 |   8765.1 |   8796.1 | 22.59 ± 0.17 |
| `range - repeats - gams anno` | 2088.1 ± 54.3 |   2032.1 |   2179.1 |  5.37 ± 0.15 |

### E5-2680 v3 RHEL 7.7

| Command                       |     Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------------------------|--------------:|---------:|---------:|------------:|
| `ctg - cds - rgr prop`        |   807.6 ± 9.9 |    790.9 |    821.8 |        1.00 |
| `ctg - cds - gams anno`       |   864.0 ± 8.0 |    847.1 |    872.2 | 1.07 ± 0.02 |
| `ctg - repeats - rgr prop`    | 1750.2 ± 25.0 |   1733.5 |   1818.1 | 2.17 ± 0.04 |
| `ctg - repeats - gams anno`   | 1856.0 ± 30.1 |   1807.6 |   1892.9 | 2.30 ± 0.05 |
| `range - cds - rgr prop`      | 4319.9 ± 85.2 |   4227.6 |   4455.6 | 5.35 ± 0.12 |
| `range - cds - gams anno`     | 1816.9 ± 24.8 |   1761.9 |   1852.8 | 2.25 ± 0.04 |
| `range - repeats - rgr prop`  | 5088.4 ± 58.3 |   5003.0 |   5158.6 | 6.30 ± 0.11 |
| `range - repeats - gams anno` | 2771.8 ± 25.3 |   2736.0 |   2819.2 | 3.43 ± 0.05 |

### Apple M2 macOS 13.4

| Command                       |     Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------------------------|--------------:|---------:|---------:|------------:|
| `ctg - cds - rgr prop`        |   498.2 ± 5.7 |    486.9 |    504.7 |        1.00 |
| `ctg - cds - gams anno`       |   512.9 ± 2.9 |    510.1 |    517.5 | 1.03 ± 0.01 |
| `ctg - repeats - rgr prop`    |  1057.5 ± 3.2 |   1053.6 |   1064.4 | 2.12 ± 0.03 |
| `ctg - repeats - gams anno`   |  1082.4 ± 5.8 |   1077.1 |   1097.5 | 2.17 ± 0.03 |
| `range - cds - rgr prop`      | 3102.5 ± 12.2 |   3085.7 |   3118.3 | 6.23 ± 0.08 |
| `range - cds - gams anno`     |  1479.7 ± 5.7 |   1471.3 |   1489.5 | 2.97 ± 0.04 |
| `range - repeats - rgr prop`  | 4221.7 ± 18.5 |   4186.9 |   4252.5 | 8.47 ± 0.10 |
| `range - repeats - gams anno` |  2013.6 ± 9.3 |   2007.8 |   2039.6 | 4.04 ± 0.05 |
