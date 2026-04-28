use crate::db::AppState;
use crate::models::OfficeHour;
use mysql::prelude::Queryable;

pub fn get_office_hours_count(state: &AppState, user_id: i64) -> Result<u64, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let count: Option<u64> = conn.exec_first(
        "SELECT COUNT(*) FROM office_working_hours WHERE user_id = ?",
        (user_id,),
    )?;

    Ok(count.unwrap_or(0))
}

pub fn insert_office_hour(
    state: &AppState,
    user_id: i64,
    day_of_week: i32,
    start_time: Option<String>,
    end_time: Option<String>,
    is_off_day: bool,
) -> Result<(), mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        "INSERT INTO office_working_hours (user_id, day_of_week, start_time, end_time, is_off_day) VALUES (?, ?, ?, ?, ?)",
        (user_id, day_of_week, start_time, end_time, is_off_day),
    )
}

pub fn get_office_hours(state: &AppState, user_id: i64) -> Result<Vec<OfficeHour>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Vec<(i32, Option<String>, Option<String>, bool)> = conn.exec(
        "SELECT day_of_week, start_time, end_time, is_off_day FROM office_working_hours WHERE user_id = ? ORDER BY day_of_week ASC",
        (user_id,),
    )?;

    Ok(result
        .into_iter()
        .map(|(day_of_week, start_time, end_time, is_off_day)| OfficeHour {
            day_of_week,
            start_time,
            end_time,
            is_off_day,
        })
        .collect())
}

pub fn upsert_office_hour(
    state: &AppState,
    user_id: i64,
    hour: &OfficeHour,
) -> Result<(), mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        r#"
        INSERT INTO office_working_hours (user_id, day_of_week, start_time, end_time, is_off_day)
        VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            start_time = VALUES(start_time),
            end_time = VALUES(end_time),
            is_off_day = VALUES(is_off_day)
        "#,
        (
            user_id,
            hour.day_of_week,
            &hour.start_time,
            &hour.end_time,
            hour.is_off_day,
        ),
    )
}
