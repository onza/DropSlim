use std::path::Path;

use super::payloads::{BatchSummaryPayload, SummaryPayload};

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

pub fn should_keep_optimized_output(
    candidate_size: u64,
    size_orig: u64,
    previous_output_size: Option<u64>,
) -> bool {
    let threshold = previous_output_size.unwrap_or(size_orig);
    candidate_size < threshold
}

pub fn build_optimize_summary_payload(
    size_orig: u64,
    size_optimized: u64,
    previous_output_size: Option<u64>,
) -> SummaryPayload {
    let new_label = format_bytes(size_optimized);

    let Some(previous) = previous_output_size else {
        if size_optimized >= size_orig {
            return SummaryPayload::AlreadyOptimized { size: new_label };
        }

        let saved = size_orig - size_optimized;
        let percent = ((100.0 / size_orig as f64) * saved as f64).round() as u64;

        return SummaryPayload::Saved {
            percent,
            from: format_bytes(size_orig),
            to: new_label,
        };
    };

    if size_optimized >= previous {
        return SummaryPayload::AlreadyOptimized { size: new_label };
    }

    let extra_saved = previous - size_optimized;
    let extra_percent = ((100.0 / previous as f64) * extra_saved as f64).round() as u64;

    if extra_percent < 1 {
        return SummaryPayload::AlreadyOptimized { size: new_label };
    }

    SummaryPayload::SavedMore {
        percent: extra_percent,
        from: format_bytes(previous),
        to: new_label,
    }
}

pub fn build_batch_summary_payload(
    total: usize,
    succeeded: usize,
    failed: usize,
    bytes_before: u64,
    bytes_after: u64,
) -> BatchSummaryPayload {
    BatchSummaryPayload {
        total: total as u32,
        succeeded: succeeded as u32,
        failed: failed as u32,
        bytes_before,
        bytes_after,
    }
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
    fn keeps_output_when_candidate_is_smaller() {
        assert!(should_keep_optimized_output(40_000, 100_000, None));
        assert!(should_keep_optimized_output(40_000, 100_000, Some(50_000)));
    }

    #[test]
    fn skips_output_when_candidate_is_larger_or_equal() {
        assert!(!should_keep_optimized_output(100_000, 100_000, None));
        assert!(!should_keep_optimized_output(120_000, 100_000, None));
        assert!(!should_keep_optimized_output(50_000, 100_000, Some(50_000)));
        assert!(!should_keep_optimized_output(55_000, 100_000, Some(50_000)));
    }

    #[test]
    fn summarizes_first_optimization() {
        assert_eq!(
            build_optimize_summary_payload(100_000, 40_000, None),
            SummaryPayload::Saved {
                percent: 60,
                from: format_bytes(100_000),
                to: format_bytes(40_000),
            }
        );
    }

    #[test]
    fn reports_no_first_pass_savings() {
        assert_eq!(
            build_optimize_summary_payload(1_000, 1_200, None),
            SummaryPayload::AlreadyOptimized {
                size: format_bytes(1_200),
            }
        );
    }

    #[test]
    fn reports_already_optimized() {
        assert_eq!(
            build_optimize_summary_payload(100_000, 50_000, Some(50_000)),
            SummaryPayload::AlreadyOptimized {
                size: format_bytes(50_000),
            }
        );
    }

    #[test]
    fn reports_additional_savings() {
        assert_eq!(
            build_optimize_summary_payload(100_000, 40_000, Some(50_000)),
            SummaryPayload::SavedMore {
                percent: 20,
                from: format_bytes(50_000),
                to: format_bytes(40_000),
            }
        );
    }

    #[test]
    fn summarizes_single_image_batch() {
        assert_eq!(
            build_batch_summary_payload(1, 1, 0, 100_000, 40_000),
            BatchSummaryPayload {
                total: 1,
                succeeded: 1,
                failed: 0,
                bytes_before: 100_000,
                bytes_after: 40_000,
            }
        );
    }

    #[test]
    fn summarizes_batch() {
        assert_eq!(
            build_batch_summary_payload(3, 3, 0, 300_000, 120_000),
            BatchSummaryPayload {
                total: 3,
                succeeded: 3,
                failed: 0,
                bytes_before: 300_000,
                bytes_after: 120_000,
            }
        );
    }
}
