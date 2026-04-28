use crate::db::AppState;
use mysql::prelude::Queryable;

pub fn find_user_for_login(
    state: &AppState,
    email_or_username: &str,
) -> Result<Option<(i64, String, String, String, String)>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_first(
        "SELECT id, username, email, password_hash, calendar_preference FROM users WHERE email = ? OR username = ?",
        (email_or_username, email_or_username),
    )
}

pub fn find_existing_user_by_email_or_username(
    state: &AppState,
    email: &str,
    username: &str,
) -> Result<Option<i64>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_first(
        "SELECT id FROM users WHERE email = ? OR username = ?",
        (email, username),
    )
}

pub fn insert_user(
    state: &AppState,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<i64, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
        (username, email, password_hash),
    )?;

    Ok(conn.last_insert_id() as i64)
}
