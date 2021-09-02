SELECT
    chr_name, COUNT(*), AVG(length)
FROM
    ctg
GROUP BY
    chr_name
