use crate::sea_generated::sea_orm_active_enums::Status;
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Weekday};
use chrono_tz::Asia::Tokyo;

/// Convert Status enum to API-friendly string representation
pub fn status_to_string(status: Status) -> String {
    match status {
        Status::Dirty => "DIRTY",
        Status::Unclean => "UNCLEAN",
        Status::Clean => "CLEAN",
        Status::Immaculate => "IMMACULATE",
        Status::Spotless => "SPOTLESS",
    }
    .to_string()
}

/// Get the current week range (Monday to Sunday) in Japan time
pub fn get_current_week_range() -> (
    chrono::DateTime<chrono_tz::Tz>,
    chrono::DateTime<chrono_tz::Tz>,
) {
    let now_jst = Tokyo.from_utc_datetime(&chrono::Utc::now().naive_utc());

    // Find the Monday of the current week
    let days_from_monday = match now_jst.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };

    let monday = now_jst - Duration::days(days_from_monday);
    let monday_start = Tokyo
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(monday.year(), monday.month(), monday.day())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .unwrap();

    let sunday_end = monday_start + Duration::days(7) - Duration::seconds(1);

    (monday_start, sunday_end)
}

/// Calculate rank change based on count vs target
pub fn calculate_rank_change(count: i64, target: i64) -> i32 {
    // Use saturating_mul to prevent overflow
    if count >= target.saturating_mul(100) {
        3 // 100x target: 3 ranks up
    } else if count >= target.saturating_mul(10) {
        2 // 10x target: 2 ranks up
    } else if count >= target {
        1 // Above target: 1 rank up
    } else {
        -1 // Below target: 1 rank down
    }
}

/// Apply rank change to status
pub fn apply_rank_change(current_status: Status, rank_change: i32) -> Status {
    let status_order = vec![
        Status::Dirty,
        Status::Unclean,
        Status::Clean,
        Status::Immaculate,
        Status::Spotless,
    ];

    let current_index = status_order
        .iter()
        .position(|s| *s == current_status)
        .unwrap_or(2); // Default to Clean
    let new_index = (current_index as i32 + rank_change).max(0).min(4) as usize;

    status_order[new_index].clone()
}

/// Status calculation logic for weekly counts
pub struct StatusCalculation {
    pub current_status: Status,
    pub new_status: Status,
    pub rank_change: i32,
    pub weekly_count: i64,
    pub target_value: i64,
}

impl StatusCalculation {
    /// Calculate new status based on weekly count and target
    pub fn calculate(current_status: Status, weekly_count: i64, target_value: i64) -> Self {
        let rank_change = calculate_rank_change(weekly_count, target_value);
        let new_status = apply_rank_change(current_status.clone(), rank_change);

        Self {
            current_status,
            new_status,
            rank_change,
            weekly_count,
            target_value,
        }
    }

    /// Get a formatted message for the status change
    pub fn get_message(&self) -> String {
        format!(
            "Status updated from {:?} to {:?} (count: {}, target: {}, change: {})",
            self.current_status,
            self.new_status,
            self.weekly_count,
            self.target_value,
            if self.rank_change > 0 {
                format!("+{}", self.rank_change)
            } else {
                self.rank_change.to_string()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_calculate_rank_change() {
        let target = 100000;

        // Test below target
        assert_eq!(calculate_rank_change(50000, target), -1);
        assert_eq!(calculate_rank_change(99999, target), -1);

        // Test at or above target (1 rank up)
        assert_eq!(calculate_rank_change(100000, target), 1);
        assert_eq!(calculate_rank_change(500000, target), 1);
        assert_eq!(calculate_rank_change(999999, target), 1);

        // Test 10x target (2 ranks up)
        assert_eq!(calculate_rank_change(1000000, target), 2);
        assert_eq!(calculate_rank_change(5000000, target), 2);
        assert_eq!(calculate_rank_change(9999999, target), 2);

        // Test 100x target (3 ranks up)
        assert_eq!(calculate_rank_change(10000000, target), 3);
        assert_eq!(calculate_rank_change(50000000, target), 3);
    }

    #[test]
    fn test_apply_rank_change() {
        // Test rank up from Clean
        assert_eq!(apply_rank_change(Status::Clean, 1), Status::Immaculate);
        assert_eq!(apply_rank_change(Status::Clean, 2), Status::Spotless);
        assert_eq!(apply_rank_change(Status::Clean, 3), Status::Spotless); // Capped at Spotless

        // Test rank down from Clean
        assert_eq!(apply_rank_change(Status::Clean, -1), Status::Unclean);
        assert_eq!(apply_rank_change(Status::Clean, -2), Status::Dirty);
        assert_eq!(apply_rank_change(Status::Clean, -3), Status::Dirty); // Capped at Dirty

        // Test boundary conditions
        assert_eq!(apply_rank_change(Status::Dirty, -1), Status::Dirty); // Can't go below Dirty
        assert_eq!(apply_rank_change(Status::Spotless, 1), Status::Spotless); // Can't go above Spotless

        // Test from other statuses
        assert_eq!(apply_rank_change(Status::Unclean, 1), Status::Clean);
        assert_eq!(apply_rank_change(Status::Immaculate, -1), Status::Clean);
    }

    #[test]
    fn test_calculate_rank_change_overflow_handling() {
        // Test with maximum i64 values to ensure overflow protection
        let max_value = i64::MAX;
        let large_target = i64::MAX / 10;

        // These calculations would overflow without saturating_mul
        assert_eq!(calculate_rank_change(max_value, large_target), 3);
        assert_eq!(calculate_rank_change(max_value, max_value), 3);

        // Test with large values that would overflow on multiplication
        let target_causing_overflow = i64::MAX / 50;
        assert_eq!(calculate_rank_change(max_value, target_causing_overflow), 3);

        // Test edge case where target * 10 would overflow
        let edge_target = i64::MAX / 11;
        assert_eq!(calculate_rank_change(max_value, edge_target), 3);

        // Test edge case where target * 100 would overflow but target * 10 wouldn't
        let edge_target_100 = i64::MAX / 101;
        assert_eq!(calculate_rank_change(max_value, edge_target_100), 3);

        // Test a case where we can verify the overflow protection works correctly
        let safe_target = 1000000;
        assert_eq!(calculate_rank_change(999999, safe_target), -1);
        assert_eq!(calculate_rank_change(1000000, safe_target), 1);
        assert_eq!(calculate_rank_change(10000000, safe_target), 2);
        assert_eq!(calculate_rank_change(100000000, safe_target), 3);
    }

    #[test]
    fn test_calculate_rank_change_boundary_values() {
        // Test boundary values and edge cases
        assert_eq!(calculate_rank_change(0, 100), -1);
        assert_eq!(calculate_rank_change(100, 0), 3);
        assert_eq!(calculate_rank_change(0, 0), 3);
        assert_eq!(calculate_rank_change(-1, 100), -1);
        assert_eq!(calculate_rank_change(100, -1), 3);
    }

    #[test]
    fn test_get_current_week_range() {
        let (start, end) = get_current_week_range();

        // Check that start is a Monday at 00:00:00
        assert_eq!(start.weekday(), Weekday::Mon);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);

        // Check that end is a Sunday at 23:59:59
        assert_eq!(end.weekday(), Weekday::Sun);
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);

        // Check that the range is exactly 7 days
        let duration = end - start;
        assert_eq!(duration.num_days(), 6); // 6 full days + 23:59:59
    }

    #[test]
    fn test_status_to_string_conversion() {
        assert_eq!(status_to_string(Status::Dirty), "DIRTY");
        assert_eq!(status_to_string(Status::Unclean), "UNCLEAN");
        assert_eq!(status_to_string(Status::Clean), "CLEAN");
        assert_eq!(status_to_string(Status::Immaculate), "IMMACULATE");
        assert_eq!(status_to_string(Status::Spotless), "SPOTLESS");
    }

    #[test]
    fn test_status_calculation() {
        let calc = StatusCalculation::calculate(Status::Clean, 150000, 100000);

        assert_eq!(calc.current_status, Status::Clean);
        assert_eq!(calc.new_status, Status::Immaculate);
        assert_eq!(calc.rank_change, 1);
        assert_eq!(calc.weekly_count, 150000);
        assert_eq!(calc.target_value, 100000);

        let message = calc.get_message();
        assert!(message.contains("Status updated from Clean to Immaculate"));
        assert!(message.contains("count: 150000"));
        assert!(message.contains("target: 100000"));
        assert!(message.contains("change: +1"));
    }
}
