SELECT
    chr_name, COUNT(*)
FROM
    ctg
GROUP BY
    chr_name
