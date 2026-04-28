use crate::db::AppState;
use crate::models::CalendarDay;
use mysql::prelude::Queryable;

pub fn get_calendar_data(
    state: &AppState,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<CalendarDay>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Vec<(String, String, Option<String>, Option<String>, bool)> = conn.exec(
        "SELECT bs_date, CAST(ad_date AS CHAR), event, tithi, holiday FROM calendar_data WHERE ad_date BETWEEN ? AND ? ORDER BY ad_date ASC",
        (start_date, end_date),
    )?;

    Ok(result
        .into_iter()
        .map(|(bs_date, ad_date, event, tithi, holiday)| CalendarDay {
            bs_date,
            ad_date,
            event,
            tithi,
            holiday,
        })
        .collect())
}
