use crate::db::AppState;
use crate::models::AttendanceRecord;
use crate::repositories::attendance_repo;

pub fn save_attendance_record(state: &AppState, record: AttendanceRecord) -> Result<bool, String> {
    let attendance_date_ad = if record.attendance_date_ad.is_empty() {
        record.date.clone()
    } else {
        record.attendance_date_ad.clone()
    };

    let existing_leave = attendance_repo::find_leave_type_on_date(state, record.user_id, &attendance_date_ad)
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(leave_type) = existing_leave {
        if leave_type == "absent" || leave_type == "public_holiday" {
            return Err("Cannot clock in to a full-day leave.".to_string());
        }

        if leave_type == "half_day" {
            if let (Some(check_in), Some(check_out)) = (&record.check_in, &record.check_out) {
                let office_info = attendance_repo::find_office_hours_for_date(state, record.user_id, &attendance_date_ad)
                    .map_err(|e| format!("Database error: {}", e))?;

                if let Some((Some(start), Some(end))) = office_info {
                    let worked = time_to_minutes(check_out) - time_to_minutes(check_in);
                    let total = time_to_minutes(&end) - time_to_minutes(&start);
                    if worked > (total / 2) + 15 {
                        return Err("Time limit exceeded for half-day leave slot.".to_string());
                    }
                }
            }
        }
    }

    attendance_repo::upsert_attendance_record(state, &record, &attendance_date_ad)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(true)
}

pub fn get_attendance_record(
    state: &AppState,
    user_id: i64,
    date: String,
) -> Result<Option<AttendanceRecord>, String> {
    attendance_repo::get_attendance_record(state, user_id, &date)
        .map_err(|e| format!("Database error: {}", e))
}

pub fn get_attendance_records(
    state: &AppState,
    user_id: i64,
    start_date: String,
    end_date: String,
) -> Result<Vec<AttendanceRecord>, String> {
    attendance_repo::get_attendance_records(state, user_id, &start_date, &end_date)
        .map_err(|e| format!("Database error: {}", e))
}

fn time_to_minutes(value: &str) -> i32 {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() == 2 {
        parts[0].parse::<i32>().unwrap_or(0) * 60 + parts[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    }
}
