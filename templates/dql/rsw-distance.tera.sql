SELECT `distance`,
       avg(gc_content) AVG_gc_content,
       avg(gc_mean)    AVG_gc_mean,
       avg(gc_stddev)  AVG_gc_stddev,
       avg(gc_cv)      AVG_gc_cv,
       avg(gc_snr)     AVG_gc_snr,
       count(ID)        COUNT
FROM rsw
GROUP BY distance
ORDER BY distance
    FORMAT TSVWithNames
