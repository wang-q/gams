pub fn mean(data: &[f32]) -> f32 {
    let len = data.len() as f32;
    let sum = data.iter().sum::<f32>();

    sum / len
}

pub fn stddev(data: &[f32]) -> f32 {
    let len = data.len() as f32;
    let mean = mean(data);

    let sq_sum = data.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>();
    (sq_sum / (len - 1.)).sqrt()
}

pub fn gc_stat(gcs: &[f32]) -> (f32, f32, f32) {
    let mean = mean(gcs);
    let stddev = stddev(gcs);

    // coefficient of variation
    let cv = if mean == 0. || mean == 1. {
        0.
    } else if mean <= 0.5 {
        stddev / mean
    } else {
        stddev / (1. - mean)
    };

    // // Signal-to-noise ratio
    // let snr = if stddev == 0. {
    //     0.
    // } else if mean <= 0.5 {
    //     mean / stddev
    // } else {
    //     (1. - mean) / stddev
    // };

    (mean, stddev, cv)
}

pub fn thresholding_algo(data: &[f32], lag: usize, threshold: f32, influence: f32) -> Vec<i32> {
    //  the results (peaks, 1 or -1)
    let mut signals: Vec<i32> = vec![0; data.len()];

    // filter out the signals (peaks) from original list (using influence arg)
    let mut filtered_data: Vec<f32> = data.to_owned();

    // the current average of the rolling window
    let mut avg_filter: Vec<f32> = vec![0.; data.len()];

    // the current standard deviation of the rolling window
    let mut std_filter: Vec<f32> = vec![0.; data.len()];

    // init avg_filter & std_filter
    avg_filter[lag - 1] = mean(&data[0..lag]);
    std_filter[lag - 1] = stddev(&data[0..lag]);

    // loop input starting at end of rolling window
    for i in lag..data.len() {
        // if the distance between the current value and average is enough standard deviations (threshold) away
        if (data[i] - avg_filter[i - 1]).abs() > threshold * std_filter[i - 1] {
            // this is a signal (i.e. peak), determine if it is a positive or negative signal
            signals[i] = if data[i] > avg_filter[i - 1] { 1 } else { -1 };

            // filter this signal out using influence
            // $filteredY[$i] = $influence * $y->[$i] + (1 - $influence) * $filteredY[$i-1];
            filtered_data[i] = influence * data[i] + (1. - influence) * filtered_data[i - 1];
        } else {
            // ensure this signal remains a zero
            signals[i] = 0;
            // ensure this value is not filtered
            filtered_data[i] = data[i];
        }

        // update average & deviation
        avg_filter[i] = mean(&filtered_data[(i - lag)..i]);
        std_filter[i] = stddev(&filtered_data[(i - lag)..i]);
    }

    signals
}

#[test]
fn thresholding_sample() {
    let input: Vec<f32> = vec![
        1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, //
        1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 1.0, //
        1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, //
        1.1, 1.0, 1.0, 1.1, 1.0, 0.8, 0.9, 1.0, 1.2, 0.9, //
        1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, //
        3.0, 2.0, 1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 3.0, 2.6, //
        4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0, 4.0, 2.0, //
        2.5, 1.0, 1.0, 1.0,
    ];
    let exp = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 1, 0, 1, 1, 1, //
        1, 1, 0, 0, 0, 0, 0, 0, 1, 1, //
        1, 1, 1, 1, 0, 0, 0, 1, 1, 1, //
        1, 0, 0, 0,
    ];
    assert_eq!(thresholding_algo(&input, 30, 5., 0.), exp);
}
