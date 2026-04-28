use crate::db::AppState;
use crate::models::OfficeHour;
use crate::repositories::office_repo;

pub fn get_office_hours(state: &AppState, user_id: i64) -> Result<Vec<OfficeHour>, String> {
    let count = office_repo::get_office_hours_count(state, user_id)
        .map_err(|e| format!("Database error: {}", e))?;

    if count == 0 {
        for day in 0..=6 {
            let (start_time, end_time, is_off_day) = if day == 6 {
                (None, None, true)
            } else {
                (Some("09:00".to_string()), Some("17:00".to_string()), false)
            };

            office_repo::insert_office_hour(state, user_id, day, start_time, end_time, is_off_day)
                .map_err(|e| format!("Database error: {}", e))?;
        }
    }

    office_repo::get_office_hours(state, user_id).map_err(|e| format!("Database error: {}", e))
}

pub fn save_office_hours(
    state: &AppState,
    user_id: i64,
    hours: Vec<OfficeHour>,
) -> Result<bool, String> {
    for hour in &hours {
        office_repo::upsert_office_hour(state, user_id, hour)
            .map_err(|e| format!("Database error: {}", e))?;
    }

    Ok(true)
}
