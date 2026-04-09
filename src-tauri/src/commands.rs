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

    let result: Option<(i64, String, String, String, String)> = conn
        .exec_first(
            "SELECT id, username, email, password_hash, calendar_preference 
         FROM users 
         WHERE email = ? OR username = ?",
            (&login_data.email_or_username, &login_data.email_or_username),
        )
        .unwrap();

    match result {
        Some((id, username, email, password_hash, calendar_preference)) => {
            if bcrypt::verify(&login_data.password, &password_hash).unwrap_or(false) {
                LoginResponse {
                    success: true,
                    message: "Login successful!".into(),
                    user: Some(UserData {
                        id,
                        username,
                        email,
                        calendar_preference,
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
            calendar_preference: "ad".into(),
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
        "SELECT id, username, email, calendar_preference FROM users WHERE id = ?",
        (user_id,),
    )
    .unwrap()
    .map(|(id, username, email, calendar_preference)| UserData {
        id,
        username,
        email,
        calendar_preference,
    })
}

#[tauri::command]
pub fn get_calendar_preference(user_id: i64, state: State<AppState>) -> Result<String, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Option<String> = conn
        .exec_first(
            "SELECT calendar_preference FROM users WHERE id = ?",
            (user_id,),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.unwrap_or_else(|| "ad".to_string()))
}

#[tauri::command]
pub fn save_calendar_preference(payload: CalendarPreferencePayload, state: State<AppState>) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let value = payload.calendar_preference.to_lowercase();
    if value != "ad" && value != "bs" {
        return Err("Invalid calendar preference. Use 'ad' or 'bs'.".to_string());
    }

    conn.exec_drop(
        "UPDATE users SET calendar_preference = ? WHERE id = ?",
        (&value, payload.user_id),
    )
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(true)
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

    let attendance_date_ad = if record.attendance_date_ad.is_empty() {
        record.date.clone()
    } else {
        record.attendance_date_ad.clone()
    };

    let existing_leave: Option<(String,)> = conn.exec_first(
        "SELECT leave_type FROM leave_logs WHERE user_id = ? AND leave_date_ad = ?",
        (record.user_id, &attendance_date_ad)
    ).unwrap_or(None);

    if let Some((leave_type,)) = existing_leave {
        if leave_type == "absent" || leave_type == "public_holiday" {
            return Err("Cannot clock in to a full-day leave.".to_string());
        } else if leave_type == "half_day" {
            if let (Some(check_in), Some(check_out)) = (&record.check_in, &record.check_out) {
                let office_info: Option<(Option<String>, Option<String>)> = conn.exec_first(
                    "SELECT start_time, end_time FROM office_working_hours WHERE user_id = ? AND day_of_week = (DAYOFWEEK(?) - 1)",
                    (record.user_id, &attendance_date_ad)
                ).unwrap_or(None);
                
                if let Some((Some(start), Some(end))) = office_info {
                    fn t2m(t: &str) -> i32 {
                        let p: Vec<&str> = t.split(':').collect();
                        if p.len() == 2 {
                            p[0].parse::<i32>().unwrap_or(0) * 60 + p[1].parse::<i32>().unwrap_or(0)
                        } else { 0 }
                    }
                    let worked = t2m(check_out) - t2m(check_in);
                    let total = t2m(&end) - t2m(&start);
                    if worked > (total / 2) + 15 {
                        return Err("Time limit exceeded for half-day leave slot.".to_string());
                    }
                }
            }
        }
    }

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
            &attendance_date_ad,
            &attendance_date_ad,
            &record.attendance_date_bs,
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

    let result: Option<(String, Option<String>, Option<String>, String, i32, bool)> = conn
        .exec_first(
            "SELECT COALESCE(attendance_date_bs, ''), check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date_ad = ?",
            (user_id, &date),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.map(
        |(attendance_date_bs, check_in, check_out, status, overtime, is_manual)| AttendanceRecord {
            user_id,
            date: date.clone(),
            attendance_date_ad: date,
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

    let result: Vec<(String, String, Option<String>, Option<String>, String, i32, bool)> = conn
        .exec(
            "SELECT CAST(attendance_date_ad AS CHAR), COALESCE(attendance_date_bs, ''), check_in, check_out, status, overtime, is_manual FROM attendance WHERE user_id = ? AND attendance_date_ad BETWEEN ? AND ? ORDER BY attendance_date_ad ASC",
            (user_id, &start_date, &end_date),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let records = result
        .into_iter()
        .map(
            |(attendance_date_ad, attendance_date_bs, check_in, check_out, status, overtime, is_manual)| AttendanceRecord {
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
        .collect();

    Ok(records)
}

#[tauri::command]
pub fn get_office_hours(user_id: i64, state: State<AppState>) -> Result<Vec<OfficeHour>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let count: Option<u64> = conn
        .exec_first(
            "SELECT COUNT(*) FROM office_working_hours WHERE user_id = ?",
            (user_id,),
        )
        .map_err(|e| format!("Database error: {}", e))?;

    if count.unwrap_or(0) == 0 {
        for day in 0..=6 {
            let (start_time, end_time, is_off_day) = if day == 6 {
                (None, None, true)
            } else {
                (Some("09:00".to_string()), Some("17:00".to_string()), false)
            };

            conn.exec_drop(
                "INSERT INTO office_working_hours (user_id, day_of_week, start_time, end_time, is_off_day) VALUES (?, ?, ?, ?, ?)",
                (user_id, day, start_time, end_time, is_off_day),
            )
            .map_err(|e| format!("Database error: {}", e))?;
        }
    }

    let result: Vec<(i32, Option<String>, Option<String>, bool)> = conn
        .exec(
            "SELECT day_of_week, start_time, end_time, is_off_day FROM office_working_hours WHERE user_id = ? ORDER BY day_of_week ASC",
            (user_id,)
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
pub fn save_office_hours(user_id: i64, hours: Vec<OfficeHour>, state: State<AppState>) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    for hour in hours {
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

    let leave_date_ad = if log.leave_date_ad.is_empty() {
        log.leave_date.clone()
    } else {
        log.leave_date_ad.clone()
    };

    let existing_attendance: Option<(Option<String>, Option<String>)> = conn.exec_first(
        "SELECT check_in, check_out FROM attendance WHERE user_id = ? AND attendance_date_ad = ?",
        (log.user_id, &leave_date_ad)
    ).unwrap_or(None);

    if let Some((check_in_opt, check_out_opt)) = existing_attendance {
        if log.leave_type == "absent" || log.leave_type == "public_holiday" {
            if check_in_opt.is_some() {
                return Err("Cannot apply for full-day leave. You have attendance logs today.".to_string());
            }
        } else if log.leave_type == "half_day" {
            if check_in_opt.is_some() && check_out_opt.is_none() {
                return Err("Please clock out first before applying for partial leave.".to_string());
            }
            if let (Some(check_in), Some(check_out)) = (check_in_opt, check_out_opt) {
                let office_info: Option<(Option<String>, Option<String>)> = conn.exec_first(
                    "SELECT start_time, end_time FROM office_working_hours WHERE user_id = ? AND day_of_week = (DAYOFWEEK(?) - 1)",
                    (log.user_id, &leave_date_ad)
                ).unwrap_or(None);
                
                if let Some((Some(start), Some(end))) = office_info {
                    fn t2m(t: &str) -> i32 {
                        let p: Vec<&str> = t.split(':').collect();
                        if p.len() == 2 {
                            p[0].parse::<i32>().unwrap_or(0) * 60 + p[1].parse::<i32>().unwrap_or(0)
                        } else { 0 }
                    }
                    let worked = t2m(&check_out) - t2m(&check_in);
                    let total = t2m(&end) - t2m(&start);
                    if worked > (total / 2) + 15 {
                        return Err("Cannot apply for half-day leave. Full day already worked.".to_string());
                    }
                }
            }
        }
    }

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
            &leave_date_ad,
            &leave_date_ad,
            &log.leave_date_bs,
            &log.leave_type,
            &log.notes,
            &log.leave_date_bs,
        ),
    )
    .map_err(|e| format!("Database error: {}", e))?;

    let id = conn.last_insert_id() as i64;

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

#[tauri::command]
pub fn get_leave_logs(user_id: i64, state: State<AppState>) -> Result<Vec<LeaveLog>, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    let result: Vec<(i64, String, String, String, String)> = conn
        .exec(
            "SELECT id, CAST(leave_date_ad AS CHAR), COALESCE(leave_date_bs, ''), leave_type, COALESCE(notes, '') FROM leave_logs WHERE user_id = ? ORDER BY leave_date_ad DESC",
            (user_id,)
        )
        .map_err(|e| format!("Database error: {}", e))?;

    let logs = result
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

    let result: Option<(i64, String, String, String, String)> = conn
        .exec_first(
            "SELECT id, CAST(leave_date_ad AS CHAR), COALESCE(leave_date_bs, ''), leave_type, COALESCE(notes, '') FROM leave_logs WHERE user_id = ? AND leave_date_ad = ?",
            (user_id, &date)
        )
        .map_err(|e| format!("Database error: {}", e))?;

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
