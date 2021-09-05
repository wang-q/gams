#!/usr/bin/env Rscript
suppressPackageStartupMessages({
    library(getopt)
    library(tidyverse)
    library(ggplot2)
    library(scales)
    library(grid)
    library(gridExtra)
    library(extrafont)
})

spec = matrix(
    c(
        "help",
        "h",
        0,
        "logical",
        "brief help message",

        "infile",
        "i",
        1,
        "character",
        "input filename",

        "xcol",
        "xcol",
        1,
        "integer",
        "column index of x",

        "ycol",
        "ycol",
        1,
        "integer",
        "column index of y",

        "yacc",
        "yacc",
        1,
        "double",
        "accuracy of y",

        "outfile",
        "o",
        1,
        "character",
        "output filename"
    ),
    byrow = TRUE,
    ncol = 5
)
opt = getopt(spec)

if (!is.null(opt$help)) {
    cat(getopt(spec, usage = TRUE))
    q(status = 1)
}

if (is.null(opt$infile)) {
    cat("--infile is need\n")
    cat(getopt(spec, usage = TRUE))
    q(status = 1)
}

if (is.null(opt$xcol)) {
    opt$xcol <- 1
}

if (is.null(opt$ycol)) {
    opt$ycol <- 2
}

if (is.null(opt$outfile)) {
    opt$outfile <- str_interp("${opt$infile}.pdf")
}

# col_type = cols() suppress the output
tbl <- read_tsv(opt$infile, col_type = cols())
colnames <- colnames(tbl)

# plot
pl_xy <- ggplot(data=tbl, aes(x=tbl[[ opt$xcol ]], y=tbl[[ opt$ycol ]])) +
    geom_line(colour="grey30") +
    geom_point(colour="grey30", fill="grey30", shape=23, size=1) +
    scale_x_continuous() +
    scale_y_continuous() +
    xlab( colnames[opt$xcol] ) + ylab( colnames[opt$ycol] ) +
    theme_bw() +
    theme(aspect.ratio=1) +
    theme(panel.grid.major.x = element_blank(), panel.grid.major.y = element_blank()) +
    theme(panel.grid.minor.x = element_blank(), panel.grid.minor.y = element_blank())

if (!is.null(opt$yacc)) {
    pl_xy <- pl_xy + scale_y_continuous(labels = scales::number_format(accuracy = opt$yacc))
}

pdf(file = opt$outfile, width = 4, height = 4, useDingbats = FALSE)
print(pl_xy)
dev.off()
