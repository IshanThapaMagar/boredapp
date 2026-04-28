use crate::db::AppState;
use crate::models::UserData;
use mysql::prelude::Queryable;

pub fn get_user_by_id(state: &AppState, user_id: i64) -> Result<Option<UserData>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Option<(i64, String, String, String)> = conn.exec_first(
        "SELECT id, username, email, calendar_preference FROM users WHERE id = ?",
        (user_id,),
    )?;

    Ok(result.map(|(id, username, email, calendar_preference)| UserData {
        id,
        username,
        email,
        calendar_preference,
    }))
}

pub fn get_calendar_preference(state: &AppState, user_id: i64) -> Result<Option<String>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_first(
        "SELECT calendar_preference FROM users WHERE id = ?",
        (user_id,),
    )
}

pub fn save_calendar_preference(
    state: &AppState,
    user_id: i64,
    calendar_preference: &str,
) -> Result<(), mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        "UPDATE users SET calendar_preference = ? WHERE id = ?",
        (calendar_preference, user_id),
    )
}
