# Change Log

## Unreleased - ReleaseDate

* Separate gars-stat
* Restructuring of documents
* Store gz seq to redis
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
