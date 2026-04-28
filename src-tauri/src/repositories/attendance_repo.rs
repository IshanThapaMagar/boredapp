use crate::db::AppState;
use crate::models::AttendanceRecord;
use mysql::prelude::Queryable;

pub fn find_leave_type_on_date(
    state: &AppState,
    user_id: i64,
    attendance_date_ad: &str,
) -> Result<Option<String>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Option<(String,)> = conn.exec_first(
        "SELECT leave_type FROM leave_logs WHERE user_id = ? AND leave_date_ad = ?",
        (user_id, attendance_date_ad),
    )?;

    Ok(result.map(|(leave_type,)| leave_type))
}

pub fn find_office_hours_for_date(
    state: &AppState,
    user_id: i64,
    date_ad: &str,
) -> Result<Option<(Option<String>, Option<String>)>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_first(
        "SELECT start_time, end_time FROM office_working_hours WHERE user_id = ? AND day_of_week = (DAYOFWEEK(?) - 1)",
        (user_id, date_ad),
    )
}

pub fn upsert_attendance_record(
    state: &AppState,
    record: &AttendanceRecord,
    attendance_date_ad: &str,
) -> Result<(), mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_drop(
        r#"
        INSERT INTO attendance (user_id, attendance_date, attendance_date_ad, attendance_date_bs, check_in, check_out, status, overtime, is_manual)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            attendance_date = VALUES(attendance_date),
            attendance_date_ad = VALUES(attendance_date_ad),
            attendance_date_bs = VALUES(attendance_date_bs),
            check_in = VALUES(check_in),
            check_out = VALUES(check_out),
            status = VALUES(status),
            overtime = VALUES(overtime),
            is_manual = VALUES(is_manual)
        "#,
        (
            record.user_id,
            attendance_date_ad,
            attendance_date_ad,
            &record.attendance_date_bs,
            &record.check_in,
            &record.check_out,
            &record.status,
            record.overtime,
            record.is_manual,
        ),
    )
}

pub fn get_attendance_record(
    state: &AppState,
    user_id: i64,
    date: &str,
) -> Result<Option<AttendanceRecord>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let result: Option<(String, Option<String>, Option<String>, String, i32, bool)> = conn.exec_first(
        "SELECT COALESCE(attendance_date_bs, ''), check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date_ad = ?",
        (user_id, date),
    )?;

    Ok(result.map(|(attendance_date_bs, check_in, check_out, status, overtime, is_manual)| {
        AttendanceRecord {
            user_id,
            date: date.to_string(),
            attendance_date_ad: date.to_string(),
            attendance_date_bs: if attendance_date_bs.is_empty() {
                None
            } else {
                Some(attendance_date_bs)
            },
            check_in,
            check_out,
            status,
            overtime,
            is_manual,
        }
    }))
}

pub fn get_attendance_records(
    state: &AppState,
    user_id: i64,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<AttendanceRecord>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    let rows: Vec<(String, String, Option<String>, Option<String>, String, i32, bool)> = conn.exec(
        "SELECT CAST(attendance_date_ad AS CHAR), COALESCE(attendance_date_bs, ''), check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date_ad BETWEEN ? AND ? ORDER BY attendance_date_ad ASC",
        (user_id, start_date, end_date),
    )?;

    Ok(rows
        .into_iter()
        .map(
            |(
                attendance_date_ad,
                attendance_date_bs,
                check_in,
                check_out,
                status,
                overtime,
                is_manual,
            )| AttendanceRecord {
                user_id,
                date: attendance_date_ad.clone(),
                attendance_date_ad,
                attendance_date_bs: if attendance_date_bs.is_empty() {
                    None
                } else {
                    Some(attendance_date_bs)
                },
                check_in,
                check_out,
                status,
                overtime,
                is_manual,
            },
        )
        .collect())
}

pub fn find_attendance_times_for_date(
    state: &AppState,
    user_id: i64,
    date_ad: &str,
) -> Result<Option<(Option<String>, Option<String>)>, mysql::Error> {
    let mut conn = state.pool.get_conn()?;

    conn.exec_first(
        "SELECT check_in, check_out FROM attendance WHERE user_id = ? AND attendance_date_ad = ?",
        (user_id, date_ad),
    )
}
