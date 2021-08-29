SELECT
    chr_name, COUNT(*), AVERAGE(length)
FROM
    ctg
GROUP BY
    chr_name
