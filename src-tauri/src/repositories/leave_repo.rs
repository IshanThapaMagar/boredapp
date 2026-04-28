use crate::db::AppState;
use crate::models::LeaveLog;
use mysql::prelude::Queryable;

pub fn upsert_leave_log(
    state: &AppState,
    log: &LeaveLog,
    leave_date_ad: &str,
) -> Result<i64, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        r#"
        INSERT INTO leave_logs (user_id, leave_date, leave_date_ad, leave_date_bs, leave_type, notes, absent_date_bs)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            leave_date = VALUES(leave_date),
            leave_date_ad = VALUES(leave_date_ad),
            leave_date_bs = VALUES(leave_date_bs),
            leave_type = VALUES(leave_type),
            notes = VALUES(notes),
            absent_date_bs = VALUES(absent_date_bs)
        "#,
        (
            log.user_id,
            leave_date_ad,
            leave_date_ad,
            &log.leave_date_bs,
            &log.leave_type,
            &log.notes,
            &log.leave_date_bs,
        ),
    )?;

    Ok(conn.last_insert_id() as i64)
}

pub fn get_leave_logs(state: &AppState, user_id: i64) -> Result<Vec<LeaveLog>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Vec<(i64, String, String, String, String)> = conn.exec(
        "SELECT id, CAST(leave_date_ad AS CHAR), COALESCE(leave_date_bs, ''), leave_type, COALESCE(notes, '') FROM leave_logs WHERE user_id = ? ORDER BY leave_date_ad DESC",
        (user_id,),
    )?;

    Ok(result
        .into_iter()
        .map(|(id, leave_date_ad, leave_date_bs, leave_type, notes)| {
            let leave_bs_value = if leave_date_bs.is_empty() {
                None
            } else {
                Some(leave_date_bs)
            };

            LeaveLog {
                id: Some(id),
                user_id,
                leave_date: leave_date_ad.clone(),
                leave_date_ad,
                leave_date_bs: leave_bs_value.clone(),
                leave_type,
                notes,
                absent_date_bs: leave_bs_value,
            }
        })
        .collect())
}

pub fn get_today_leave(
    state: &AppState,
    user_id: i64,
    date: &str,
) -> Result<Option<LeaveLog>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Option<(i64, String, String, String, String)> = conn.exec_first(
        "SELECT id, CAST(leave_date_ad AS CHAR), COALESCE(leave_date_bs, ''), leave_type, COALESCE(notes, '') FROM leave_logs WHERE user_id = ? AND leave_date_ad = ?",
        (user_id, date),
    )?;

    Ok(result.map(|(id, leave_date_ad, leave_date_bs, leave_type, notes)| {
        let leave_bs_value = if leave_date_bs.is_empty() {
            None
        } else {
            Some(leave_date_bs)
        };

        LeaveLog {
            id: Some(id),
            user_id,
            leave_date: leave_date_ad.clone(),
            leave_date_ad,
            leave_date_bs: leave_bs_value.clone(),
            leave_type,
            notes,
            absent_date_bs: leave_bs_value,
        }
    }))
}

pub fn remove_leave_log(state: &AppState, leave_id: i64) -> Result<(), mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop("DELETE FROM leave_logs WHERE id = ?", (leave_id,))
}
