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

        "outfile",
        "o",
        1,
        "character",
        "output filename",

        "lag",
        "1",
        1,
        "integer",
        "how much data will be smoothed",

        "influence",
        "2",
        1,
        "integer",
        "the influence of signals on the algorithm's detection threshold",

        "threshold",
        "3",
        1,
        "integer",
        "the number of standard deviations from the moving mean"
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

if (is.null(opt$lag)) {
    opt$lag <- 1000
}

if (is.null(opt$influence)) {
    opt$influence <- 20
}

if (is.null(opt$threshold)) {
    opt$threshold <- 3
}

# col_type = cols() suppress the output
tbl <- read_tsv(opt$infile, col_type = cols())

# https://stackoverflow.com/questions/22583391/peak-signal-detection-in-realtime-timeseries-data/54507329#54507329
ThresholdingAlgo <- function(y, lag, threshold, influence) {
    signals <- rep(0, length(y))
    filteredY <- y[0:lag]
    avgFilter <- NULL
    stdFilter <- NULL
    avgFilter[lag] <- mean(y[0:lag], na.rm = TRUE)
    stdFilter[lag] <- sd(y[0:lag], na.rm = TRUE)
    for (i in (lag + 1):length(y)) {
        if (abs(y[i] - avgFilter[i - 1]) > threshold * stdFilter[i - 1]) {
            if (y[i] > avgFilter[i - 1]) {
                signals[i] <- 1
            } else {
                signals[i] <- -1
            }
            filteredY[i] <- influence * y[i] + (1 - influence) * filteredY[i - 1]
        } else {
            signals[i] <- 0
            filteredY[i] <- y[i]
        }
        avgFilter[i] <- mean(filteredY[(i - lag):i], na.rm = TRUE)
        stdFilter[i] <- sd(filteredY[(i - lag):i], na.rm = TRUE)
    }
    return(list("signals" = signals, "avgFilter" = avgFilter, "stdFilter" = stdFilter))
}

y <- tbl$gc_content
result <- ThresholdingAlgo(y, opt$lag, opt$threshold, opt$influence)
tbl$signal <- result$signal

# outputs
if (is.null(opt$outfile)) {
    cat(format_tsv(tbl))
} else {
    write_tsv(tbl, file=opt$outfile)
}
figfile <- if (is.null(opt$outfile)) {
    "stdout.pdf"
} else {
    str_c(opt$outfile, ".pdf")
}

# Plot result
write(figfile, stderr())
pdf(file=figfile, useDingbats=FALSE)
par(mfrow = c(2, 1), oma = c(2, 2, 0, 0) + 0.1, mar = c(0, 0, 2, 1) + 0.2)
plot(1:length(y), y, type = "l", ylab = "", xlab = "")
lines(1:length(y), result$avgFilter, type = "l", col = "cyan", lwd = 2)
lines(1:length(y), result$avgFilter + opt$threshold * result$stdFilter, type = "l", col = "green", lwd = 2)
lines(1:length(y), result$avgFilter - opt$threshold * result$stdFilter, type = "l", col = "green", lwd = 2)
plot(result$signals, type = "S", col = "red", ylab = "", xlab = "", ylim = c(-1.5, 1.5), lwd = 2)
dev.off()
