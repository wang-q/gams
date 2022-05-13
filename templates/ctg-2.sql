SELECT
    chr_id, COUNT(*), AVG(length)
FROM
    ctg
GROUP BY
    chr_id
