use std::path::Path;

pub fn file_size(path: &Path) -> std::io::Result<u64> {
    Ok(std::fs::metadata(path)?.len())
}

pub fn format_bytes(bytes: u64) -> String {
    let use_decimal = cfg!(target_os = "macos");
    let unit: u64 = if use_decimal { 1000 } else { 1024 };

    if bytes < unit {
        return format!("{bytes} B");
    }

    if bytes < unit * unit {
        let value = bytes as f64 / unit as f64;

        if use_decimal {
            return format!("{} KB", value.round());
        }

        return format!("{value:.1} KB");
    }

    let value = bytes as f64 / (unit as f64 * unit as f64);
    format!("{value:.1} MB")
}

pub fn build_optimize_summary(
    size_orig: u64,
    size_optimized: u64,
    previous_output_size: Option<u64>,
) -> String {
    let new_label = format_bytes(size_optimized);

    let Some(previous) = previous_output_size else {
        if size_optimized >= size_orig {
            return format!("Already optimized · {new_label}");
        }

        let saved = size_orig - size_optimized;
        let percent = ((100.0 / size_orig as f64) * saved as f64).round() as u64;

        return format!(
            "You saved {}% · {} → {}",
            percent,
            format_bytes(size_orig),
            new_label
        );
    };

    if size_optimized >= previous {
        return format!("Already optimized · {new_label}");
    }

    let extra_saved = previous - size_optimized;
    let extra_percent = ((100.0 / previous as f64) * extra_saved as f64).round() as u64;

    if extra_percent < 1 {
        return format!("Already optimized · {new_label}");
    }

    format!(
        "Saved {}% more · {} → {}",
        extra_percent,
        format_bytes(previous),
        new_label
    )
}

pub fn build_batch_summary(
    total: usize,
    succeeded: usize,
    failed: usize,
    bytes_before: u64,
    bytes_after: u64,
) -> String {
    let image_label = if total == 1 {
        "1 image".to_string()
    } else {
        format!("{total} images")
    };

    let mut parts = vec![image_label];

    if failed > 0 {
        parts.push(format!("{failed} failed"));
    }

    if succeeded > 0 {
        let saved = bytes_before.saturating_sub(bytes_after);

        if saved > 0 {
            let percent = ((100.0 / bytes_before as f64) * saved as f64).round() as u64;
            parts.push(format!("saved {} ({percent}%)", format_bytes(saved)));
        } else if failed == 0 {
            parts.push("already optimized".to_string());
        }
    }

    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_below_one_kilobyte() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn uses_decimal_kilobytes_on_macos() {
        assert_eq!(format_bytes(1500), "2 KB");
        assert_eq!(format_bytes(100_000), "100 KB");
        assert_eq!(format_bytes(2_500_000), "2.5 MB");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn uses_binary_kilobytes_off_macos() {
        assert_eq!(format_bytes(1500), "1.5 KB");
    }

    #[test]
    fn summarizes_first_optimization() {
        assert_eq!(
            build_optimize_summary(100_000, 40_000, None),
            "You saved 60% · 100 KB → 40 KB"
        );
    }

    #[test]
    fn reports_no_first_pass_savings() {
        assert_eq!(
            build_optimize_summary(1_000, 1_200, None),
            "Already optimized · 1 KB"
        );
    }

    #[test]
    fn reports_already_optimized() {
        assert_eq!(
            build_optimize_summary(100_000, 50_000, Some(50_000)),
            "Already optimized · 50 KB"
        );
    }

    #[test]
    fn reports_additional_savings() {
        assert_eq!(
            build_optimize_summary(100_000, 40_000, Some(50_000)),
            "Saved 20% more · 50 KB → 40 KB"
        );
    }

    #[test]
    fn summarizes_single_image_batch() {
        assert_eq!(
            build_batch_summary(1, 1, 0, 100_000, 40_000),
            "1 image · saved 60 KB (60%)"
        );
    }

    #[test]
    fn summarizes_batch() {
        assert_eq!(
            build_batch_summary(3, 3, 0, 300_000, 120_000),
            "3 images · saved 180 KB (60%)"
        );
    }
}
