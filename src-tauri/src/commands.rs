use mysql::prelude::Queryable;
use tauri::State;
use crate::db::AppState;
use crate::models::*;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You're ready to get started with the app!", name)
}

#[tauri::command]
pub fn login(login_data: LoginRequest, state: State<AppState>) -> LoginResponse {
    let mut conn = state.pool.get_conn().unwrap();

    let result: Option<(i64, String, String, String)> = conn
        .exec_first(
            "SELECT id, username, email, password_hash 
         FROM users 
         WHERE email = ? OR username = ?",
            (&login_data.email_or_username, &login_data.email_or_username),
        )
        .unwrap();

    match result {
        Some((id, username, email, password_hash)) => {
            if bcrypt::verify(&login_data.password, &password_hash).unwrap_or(false) {
                LoginResponse {
                    success: true,
                    message: "Login successful!".into(),
                    user: Some(UserData {
                        id,
                        username,
                        email,
                    }),
                }
            } else {
                LoginResponse {
                    success: false,
                    message: "Invalid email/username or password".into(),
                    user: None,
                }
            }
        }
        None => LoginResponse {
            success: false,
            message: "Invalid email/username or password".into(),
            user: None,
        },
    }
}

#[tauri::command]
pub fn register(user: User, state: State<AppState>) -> LoginResponse {
    let mut conn = state.pool.get_conn().unwrap();

    let exists: Option<i64> = conn
        .exec_first(
            "SELECT id FROM users WHERE email = ? OR username = ?",
            (&user.email, &user.username),
        )
        .unwrap();

    if exists.is_some() {
        return LoginResponse {
            success: false,
            message: "User with this email or username already exists".into(),
            user: None,
        };
    }

    let password_hash = match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return LoginResponse {
                success: false,
                message: "Error processing registration".into(),
                user: None,
            }
        }
    };

    conn.exec_drop(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
        (&user.username, &user.email, &password_hash),
    )
    .unwrap();

    let user_id = conn.last_insert_id() as i64;

    LoginResponse {
        success: true,
        message: "Registration successful!".into(),
        user: Some(UserData {
            id: user_id,
            username: user.username,
            email: user.email,
        }),
    }
}

#[tauri::command]
pub fn logout() -> bool {
    true
}

#[tauri::command]
pub fn get_user_by_id(user_id: i64, state: State<AppState>) -> Option<UserData> {
    let mut conn = state.pool.get_conn().unwrap();

    conn.exec_first(
        "SELECT id, username, email FROM users WHERE id = ?",
        (user_id,),
    )
    .unwrap()
    .map(|(id, username, email)| UserData {
        id,
        username,
        email,
    })
}

#[tauri::command]
pub fn save_attendance_record(
    record: AttendanceRecord,
    state: State<AppState>,
) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.exec_drop(
        r#"
        INSERT INTO attendance (user_id, attendance_date, check_in, check_out, status, overtime, is_manual)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE 
            check_in = VALUES(check_in),
            check_out = VALUES(check_out),
            status = VALUES(status),
            overtime = VALUES(overtime),
            is_manual = VALUES(is_manual)
        "#,
        (
            record.user_id,
            &record.date,
            record.check_in,
            record.check_out,
            &record.status,
            record.overtime,
            record.is_manual,
        ),
    )
    .map(|_| true)
    .map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
pub fn get_attendance_record(
    user_id: i64,
    date: String,
    state: State<AppState>,
) -> Result<Option<AttendanceRecord>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Option<(Option<String>, Option<String>, String, i32, bool)> = conn
        .exec_first(
            "SELECT check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date = ?",
            (user_id, &date),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.map(
        |(check_in, check_out, status, overtime, is_manual)| AttendanceRecord {
            user_id,
            date,
            check_in,
            check_out,
            status,
            overtime,
            is_manual,
        },
    ))
}

#[tauri::command]
pub fn get_attendance_records(
    user_id: i64,
    start_date: String,
    end_date: String,
    state: State<AppState>,
) -> Result<Vec<AttendanceRecord>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Vec<(String, Option<String>, Option<String>, String, i32, bool)> = conn
        .exec(
            "SELECT CAST(attendance_date AS CHAR), check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date BETWEEN ? AND ? ORDER BY attendance_date ASC",
            (user_id, &start_date, &end_date),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let records = result
        .into_iter()
        .map(
            |(date, check_in, check_out, status, overtime, is_manual)| AttendanceRecord {
                user_id,
                date,
                check_in,
                check_out,
                status,
                overtime,
                is_manual,
            },
        )
        .collect();

    Ok(records)
}

#[tauri::command]
pub fn get_office_hours(state: State<AppState>) -> Result<Vec<OfficeHour>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Vec<(i32, Option<String>, Option<String>, bool)> = conn
        .query(
            "SELECT day_of_week, start_time, end_time, is_off_day FROM office_working_hours ORDER BY day_of_week ASC"
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let hours = result
        .into_iter()
        .map(
            |(day_of_week, start_time, end_time, is_off_day)| OfficeHour {
                day_of_week,
                start_time,
                end_time,
                is_off_day,
            },
        )
        .collect();

    Ok(hours)
}

#[tauri::command]
pub fn save_office_hours(hours: Vec<OfficeHour>, state: State<AppState>) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    for hour in hours {
        conn.exec_drop(
            r#"
            INSERT INTO office_working_hours (day_of_week, start_time, end_time, is_off_day)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE 
                start_time = VALUES(start_time),
                end_time = VALUES(end_time),
                is_off_day = VALUES(is_off_day)
            "#,
            (
                hour.day_of_week,
                hour.start_time,
                hour.end_time,
                hour.is_off_day,
            ),
        )
        .map_err(|e| format!("Database error: {}", e))?;
    }

    Ok(true)
}

#[tauri::command]
pub fn add_leave_log(log: LeaveLog, state: State<AppState>) -> Result<LeaveLog, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.exec_drop(
        r#"
        INSERT INTO leave_logs (user_id, leave_date, leave_type, notes, absent_date_bs)
        VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE 
            leave_type = VALUES(leave_type),
            notes = VALUES(notes),
            absent_date_bs = VALUES(absent_date_bs)
        "#,
        (
            log.user_id,
            &log.leave_date,
            &log.leave_type,
            &log.notes,
            &log.absent_date_bs,
        ),
    )
    .map_err(|e| format!("Database error: {}", e))?;

    let id = conn.last_insert_id() as i64;

    Ok(LeaveLog {
        id: Some(id),
        user_id: log.user_id,
        leave_date: log.leave_date,
        leave_type: log.leave_type,
        notes: log.notes,
        absent_date_bs: log.absent_date_bs,
    })
}

#[tauri::command]
pub fn get_leave_logs(user_id: i64, state: State<AppState>) -> Result<Vec<LeaveLog>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Vec<(i64, String, String, String, Option<String>)> = conn
        .exec(
            "SELECT id, CAST(leave_date AS CHAR), leave_type, COALESCE(notes, ''), absent_date_bs FROM leave_logs WHERE user_id = ? ORDER BY leave_date DESC",
            (user_id,)
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let logs = result
        .into_iter()
        .map(
            |(id, leave_date, leave_type, notes, absent_date_bs)| LeaveLog {
                id: Some(id),
                user_id,
                leave_date,
                leave_type,
                notes,
                absent_date_bs,
            },
        )
        .collect();

    Ok(logs)
}

#[tauri::command]
pub fn get_today_leave(
    user_id: i64,
    date: String,
    state: State<AppState>,
) -> Result<Option<LeaveLog>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Option<(i64, String, String, String, Option<String>)> = conn
        .exec_first(
            "SELECT id, CAST(leave_date AS CHAR), leave_type, COALESCE(notes, ''), absent_date_bs FROM leave_logs WHERE user_id = ? AND leave_date = ?",
            (user_id, &date)
        )
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.map(
        |(id, leave_date, leave_type, notes, absent_date_bs)| LeaveLog {
            id: Some(id),
            user_id,
            leave_date,
            leave_type,
            notes,
            absent_date_bs,
        },
    ))
}

#[tauri::command]
pub fn remove_leave_log(data: LeaveLogRequest, state: State<AppState>) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.exec_drop("DELETE FROM leave_logs WHERE id = ?", (data.id,))
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn get_calendar_data(
    start_date: String,
    end_date: String,
    state: State<AppState>,
) -> Result<Vec<CalendarDay>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Vec<(String, String, Option<String>, Option<String>, bool)> = conn
        .exec(
            "SELECT bs_date, CAST(ad_date AS CHAR), event, tithi, holiday FROM calendar_data WHERE ad_date BETWEEN ? AND ? ORDER BY ad_date ASC",
            (&start_date, &end_date),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let days = result
        .into_iter()
        .map(|(bs_date, ad_date, event, tithi, holiday)| CalendarDay {
            bs_date,
            ad_date,
            event,
            tithi,
            holiday,
        })
        .collect();

    Ok(days)
}
