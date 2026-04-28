use crate::db::AppState;
use crate::models::{LeaveLog, LeaveLogRequest};
use crate::repositories::{attendance_repo, leave_repo};

pub fn add_leave_log(state: &AppState, log: LeaveLog) -> Result<LeaveLog, String> {
    let leave_date_ad = if log.leave_date_ad.is_empty() {
        log.leave_date.clone()
    } else {
        log.leave_date_ad.clone()
    };

    let existing_attendance = attendance_repo::find_attendance_times_for_date(state, log.user_id, &leave_date_ad)
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some((check_in_opt, check_out_opt)) = existing_attendance {
        if (log.leave_type == "absent" || log.leave_type == "public_holiday") && check_in_opt.is_some() {
            return Err("Cannot apply for full-day leave. You have attendance logs today.".to_string());
        }

        if log.leave_type == "half_day" {
            if check_in_opt.is_some() && check_out_opt.is_none() {
                return Err("Please clock out first before applying for partial leave.".to_string());
            }

            if let (Some(check_in), Some(check_out)) = (check_in_opt, check_out_opt) {
                let office_info = attendance_repo::find_office_hours_for_date(state, log.user_id, &leave_date_ad)
                    .map_err(|e| format!("Database error: {}", e))?;

                if let Some((Some(start), Some(end))) = office_info {
                    let worked = time_to_minutes(&check_out) - time_to_minutes(&check_in);
                    let total = time_to_minutes(&end) - time_to_minutes(&start);
                    if worked > (total / 2) + 15 {
                        return Err("Cannot apply for half-day leave. Full day already worked.".to_string());
                    }
                }
            }
        }
    }

    let id = leave_repo::upsert_leave_log(state, &log, &leave_date_ad)
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(LeaveLog {
        id: Some(id),
        user_id: log.user_id,
        leave_date: leave_date_ad.clone(),
        leave_date_ad,
        leave_date_bs: log.leave_date_bs,
        leave_type: log.leave_type,
        notes: log.notes,
        absent_date_bs: log.absent_date_bs,
    })
}

pub fn get_leave_logs(state: &AppState, user_id: i64) -> Result<Vec<LeaveLog>, String> {
    leave_repo::get_leave_logs(state, user_id).map_err(|e| format!("Database error: {}", e))
}

pub fn get_today_leave(state: &AppState, user_id: i64, date: String) -> Result<Option<LeaveLog>, String> {
    leave_repo::get_today_leave(state, user_id, &date).map_err(|e| format!("Database error: {}", e))
}

pub fn remove_leave_log(state: &AppState, data: LeaveLogRequest) -> Result<bool, String> {
    leave_repo::remove_leave_log(state, data.id).map_err(|e| format!("Database error: {}", e))?;
    Ok(true)
}

fn time_to_minutes(value: &str) -> i32 {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() == 2 {
        parts[0].parse::<i32>().unwrap_or(0) * 60 + parts[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    }
}
