SELECT '_TABLE_'                       TABLE,
       '_COLUMN_'                      COLUMN,
       count(`ID`)                     COUNT,
       round(avg(`_COLUMN_`), 4)       AVG,
       round(stddevPop(`_COLUMN_`), 4) STDDEV,
       min(`_COLUMN_`)                 MIN,
       max(`_COLUMN_`)                 MAX,
       sum(`_COLUMN_`)                 SUM
FROM _TABLE_
    FORMAT TSVWithNames
