use intspan::IntSpan;

pub fn center_sw(
    parent: &IntSpan,
    start: i32,
    end: i32,
    size: i32,
    max: i32,
) -> Vec<(IntSpan, String, i32)> {
    let mut windows = vec![];

    let w0 = center_resize(parent, &IntSpan::from_pair(start, end), size);
    windows.push((w0.clone(), "M".to_string(), 0));

    for sw_type in ["L", "R"] {
        // sw_start and sw_end are both index of parent
        let mut sw_start;
        let mut sw_end;

        if sw_type == "R" {
            sw_start = parent.index(w0.max()) + 1;
            sw_end = sw_start + size - 1;
        } else {
            sw_end = parent.index(w0.min()) - 1;
            sw_start = sw_end - size + 1;
        }

        // distance is from 1 to max
        for sw_distance in 1..=max {
            if sw_start < 1 {
                break;
            }
            if sw_end > parent.size() {
                break;
            }

            let sw_intspan = parent.slice(sw_start, sw_end);

            if sw_intspan.size() < size {
                break;
            }

            windows.push((sw_intspan.clone(), sw_type.to_string(), sw_distance));

            if sw_type == "R" {
                sw_start = sw_end + 1;
                sw_end = sw_start + size - 1;
            } else {
                sw_end = sw_start - 1;
                sw_start = sw_end - size + 1;
            }
        }
    }

    windows
}

#[test]
fn test_center_sw() {
    // parent, start, end, exp
    let tests = vec![
        ("1-9999", 500, 500, ("451-549", "M", 0, 3)),
        ("1-9999", 500, 800, ("600-699", "M", 0, 3)),
        ("1-9999", 101, 101, ("52-150", "M", 0, 2)),
        ("10001-19999", 10101, 10101, ("10052-10150", "M", 0, 2)),
    ];

    for (parent, start, end, exp) in tests {
        let windows = center_sw(&IntSpan::from(parent), start, end, 100, 1);

        assert_eq!(windows[0].0.to_string(), exp.0);
        assert_eq!(windows[0].1, exp.1);
        assert_eq!(windows[0].2, exp.2);
        assert_eq!(windows.len(), exp.3);
    }
}

pub fn sliding(intspan: &IntSpan, size: i32, step: i32) -> Vec<IntSpan> {
    let mut windows = vec![];

    let mut start = 1;
    loop {
        let end = start + size - 1;
        if end > intspan.size() {
            break;
        }
        let window = intspan.slice(start, end);
        start += step;

        windows.push(window);
    }

    windows
}

pub fn center_resize(parent: &IntSpan, intspan: &IntSpan, resize: i32) -> IntSpan {
    // find the middles of intspan
    let half_size = intspan.size() / 2;
    let mid_left = if half_size == 0 {
        intspan.at(1)
    } else {
        intspan.at(half_size)
    };
    let mid_right = if half_size == 0 {
        intspan.at(1)
    } else {
        intspan.at(half_size + 1)
    };
    let mid_left_idx = parent.index(mid_left);
    let mid_right_idx = parent.index(mid_right);

    // map to parent
    let half_resize = resize / 2;
    let mut left_idx = mid_left_idx - half_resize + 1;
    if left_idx < 1 {
        left_idx = 1;
    }
    let mut right_idx = mid_right_idx + half_resize - 1;
    if right_idx > parent.size() {
        right_idx = parent.size();
    }

    parent.slice(left_idx, right_idx)
}

#[test]
fn test_center_resize() {
    // parent, runlist, resize, exp
    let tests = vec![
        ("1-500", "201", 100, "152-250"),
        ("1-500", "200", 100, "151-249"),
        ("1-500", "200-201", 100, "151-250"),
        ("1-500", "199-201", 100, "150-249"),
        ("1-500", "199-202", 100, "151-250"),
        ("1-500", "100-301", 100, "151-250"),
        ("1-500", "1", 100, "1-50"),
        ("1-500", "500", 100, "451-500"),
        ("1001-1500", "1200-1201", 100, "1151-1250"),
    ];

    for (parent, runlist, resize, exp) in tests {
        let intspan = IntSpan::from(runlist);
        let resized = center_resize(&IntSpan::from(parent), &intspan, resize);

        assert_eq!(resized.to_string(), exp);
    }
}
