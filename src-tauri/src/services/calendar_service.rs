use crate::db::AppState;
use crate::models::CalendarDay;
use crate::repositories::calendar_repo;

pub fn get_calendar_data(
    state: &AppState,
    start_date: String,
    end_date: String,
) -> Result<Vec<CalendarDay>, String> {
    calendar_repo::get_calendar_data(state, &start_date, &end_date)
        .map_err(|e| format!("Database error: {}", e))
}
