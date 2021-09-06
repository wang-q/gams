SELECT '_TABLE_'                              TABLE,
       `type`                                 TYPE,
       count(`ID`)                            COUNT,
       round(avg(chr_end - chr_start + 1), 2) AVG_length,
       sum(chr_end - chr_start + 1)           SUM_length
FROM _TABLE_
GROUP BY `type`
ORDER BY `type`
    FORMAT TSVWithNames
