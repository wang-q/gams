CREATE TABLE ctg
(
    `ID` FixedString(64),
    `chr_name` FixedString(16),
    `chr_start` UInt32,
    `chr_end` UInt32,
    `chr_strand` FixedString(1),
    `length` UInt32
)
ENGINE = MergeTree()
PRIMARY KEY (`ID`)
ORDER BY (`ID`, `chr_name`, `chr_start`, `chr_end`);
