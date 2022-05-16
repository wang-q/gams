SELECT `tag`,
       `distance`,
       round(avg(gc_content), 4) AVG_gc_content,
       round(avg(gc_mean), 4)    AVG_gc_mean,
       round(avg(gc_stddev), 4)  AVG_gc_stddev,
       round(avg(gc_cv), 4)      AVG_gc_cv,
       count(ID)       COUNT
FROM fsw
GROUP BY tag, distance
ORDER BY tag, distance
    FORMAT TSVWithNames
