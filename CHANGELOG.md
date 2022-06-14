# Change Log

## Unreleased - ReleaseDate

## 0.3.1 - 2022-06-15

* Add `gars anno`
* Add `--range` to `gars tsv`

* Switch to `clap` v3.2
* Switch to `intspan` v0.7.1
* Remove TLS
* Remove SNR from gc_stat()

* Rename .ranges to .rg

* Add more tests
* Add more benchmarks

## 0.3.0 - 2022-05-13

* Rename `gars range` to `gars feature`
* Rename `gars rsw` to `gars fsw`
* Rename `gars pos` to `gars range`
* Rename `gars wave` to `gars peak`
* Add `gars locate`
* Add `gars clear`

* Avoid get_scan_vec() inside loops
    * Speedup `gars feature`
    * Speedup `gars fsw`
    * Speedup `gars range`
    * Speedup `gars peak`

* Rename .pos.txt to .ranges
* Rename chr_name to chr_id

* `redis.rs`
    * Rename find_one() to find_one_z()
    * Add find_one_l()
    * Add build_idx_ctg()
    * Add get_idx_ctg()
    * Add find_one_idx()
    * Add get_vec_chr()
    * Add get_vec_ctg()
    * Add get_vec_feature()
    * Add get_vec_peak()

* Add more tests

## 0.2.1 - 2022-05-10

* Separate gars-stat
* Restructuring of documents
* Store gzipped seq to redis
* cache_gc_content and cache_gc_stat()

## 0.2.0 - 2022-04-23

* Renamed to `gars`

* Add `gars status stop`

* Queries done by clickhouse
    * `gars env` creates sqls
    * Stats of ctg and rsw

* `gars stat`
    * `Polars` can't read sqls, so use the built-in queries
    * `Datafusion` makes the compilation extremely slow

## 0.1.0 - 2021-08-27

* Skeletons, need to be filled

* Subcommands
    * env
    * gen
    * pos
    * range
    * rsw
    * sliding
    * status
    * tsv
    * wave
