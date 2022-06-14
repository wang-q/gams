CREATE TABLE fsw
(
    `ID`          String,
    `chr_id`      String,
    `chr_start`   UInt32,
    `chr_end`     UInt32,
    `type`        String,
    `distance`    UInt32,
    `tag`         String,
    `gc_content`  Float32,
    `gc_mean`     Float32,
    `gc_stddev`   Float32,
    `gc_cv`       Float32,
    `cdsProp`     Float32,
    `repeatsProp` Float32
)
    ENGINE = MergeTree()
        PRIMARY KEY (`ID`)
        ORDER BY (`ID`, `chr_id`, `chr_start`, `chr_end`);
