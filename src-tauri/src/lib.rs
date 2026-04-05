use mysql::prelude::Queryable;
use mysql::Pool;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use bcrypt;

// User structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    pub password: String,
}

// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
    pub remember_me: bool,
}

// Login response structure
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserData>,
}

// User data (without password)
#[derive(Debug, Clone, Serialize)]
pub struct UserData {
    pub id: i64,
    pub username: String,
    pub email: String,
}

// Attendance record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub user_id: i64,
    pub date: String,              // YYYY-MM-DD
    pub check_in: Option<String>,  // HH:mm
    pub check_out: Option<String>, // HH:mm
    pub status: String,
    pub overtime: i32,
    pub is_manual: bool,
}

// Office Hour structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeHour {
    pub day_of_week: i32,           // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub start_time: Option<String>, // HH:mm
    pub end_time: Option<String>,   // HH:mm
    pub is_off_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveLog {
    pub id: Option<i64>,
    pub user_id: i64,
    pub leave_date: String, // YYYY-MM-DD
    pub leave_type: String, // 'public_holiday' or 'absent'
    pub notes: String,
    pub absent_date_bs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDay {
    pub bs_date: String,
    pub ad_date: String,
    pub event: Option<String>,
    pub tithi: Option<String>,
    pub holiday: bool,
}

#[derive(Debug, Deserialize)]
pub struct LeaveLogRequest {
    pub id: i64,
}

// Application state with database connection
pub struct AppState {
    pool: Pool,
}

impl AppState {
    pub fn new(database_url: &str) -> Result<Self, mysql::Error> {
        let pool = Pool::new(database_url)?;

        let mut conn = pool.get_conn()?;

        // Create users table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )?;

        // Create attendance table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS attendance (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                attendance_date DATE NOT NULL,
                check_in VARCHAR(8),
                check_out VARCHAR(8),
                status VARCHAR(20),
                overtime INT DEFAULT 0,
                is_manual BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE KEY user_date (user_id, attendance_date),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )?;

        // Migration: Ensure attendance_date column exists (if table was created before this change)
        let column_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'attendance' AND COLUMN_NAME = 'attendance_date' AND TABLE_SCHEMA = DATABASE()"
        )?;

        if column_exists.is_none() {
            // This is a safety measure. If we reach here, we need to alter.
            conn.query_drop(
                "ALTER TABLE attendance ADD COLUMN attendance_date DATE NOT NULL AFTER user_id",
            )?;
            conn.query_drop(
                "ALTER TABLE attendance UNIQUE KEY user_date (user_id, attendance_date)",
            )?;
        }

        // Create office_working_hours table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS office_working_hours (
                day_of_week INT PRIMARY KEY,
                start_time VARCHAR(8),
                end_time VARCHAR(8),
                is_off_day BOOLEAN DEFAULT FALSE
            )
            "#,
        )?;

        // Initialize default office working hours if empty
        let count: Option<u64> = conn.query_first("SELECT COUNT(*) FROM office_working_hours")?;
        if count.unwrap_or(0) == 0 {
            for day in 0..=6 {
                let (start_time, end_time, is_off_day) = if day == 6 {
                    // Saturday is an off day by default
                    (None, None, true)
                } else {
                    // Monday to Sunday (except Saturday) default working hours
                    (Some("09:00".to_string()), Some("17:00".to_string()), false)
                };

                conn.exec_drop(
                    "INSERT INTO office_working_hours (day_of_week, start_time, end_time, is_off_day) VALUES (?, ?, ?, ?)",
                    (day, start_time, end_time, is_off_day)
                )?;
            }
        }

        // Create leave_logs table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS leave_logs (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                leave_date DATE NOT NULL,
                leave_type VARCHAR(50) NOT NULL,
                notes TEXT,
                absent_date_bs VARCHAR(20),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE KEY user_leave_date (user_id, leave_date),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )?;

        let bs_column_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'leave_logs' AND COLUMN_NAME = 'absent_date_bs' AND TABLE_SCHEMA = DATABASE()"
        )?;

        if bs_column_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE leave_logs ADD COLUMN absent_date_bs VARCHAR(20) AFTER notes",
            )?;
        }

        // Create calendar_data table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS calendar_data (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                bs_date VARCHAR(255) NOT NULL,
                ad_date DATE NOT NULL UNIQUE,
                event TEXT,
                tithi VARCHAR(255),
                holiday BOOLEAN DEFAULT FALSE
            )
            "#,
        )?;

        // Insert demo user if empty
        let count: Option<u64> = conn.query_first("SELECT COUNT(*) FROM users")?;

        if count.unwrap_or(0) == 0 {
            let admin_hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap();
            conn.exec_drop(
                "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
                ("admin", "admin@example.com", admin_hash),
            )?;
        }

        Ok(Self { pool })
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You're ready to get started with the app!", name)
}

#[tauri::command]
fn login(login_data: LoginRequest, state: State<AppState>) -> LoginResponse {
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
fn register(user: User, state: State<AppState>) -> LoginResponse {
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
fn logout() -> bool {
    // Clear session/token in production
    true
}

#[tauri::command]
fn get_user_by_id(user_id: i64, state: State<AppState>) -> Option<UserData> {
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
fn save_attendance_record(
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
fn get_attendance_record(
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
fn get_attendance_records(
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
fn get_office_hours(state: State<AppState>) -> Result<Vec<OfficeHour>, String> {
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
fn save_office_hours(hours: Vec<OfficeHour>, state: State<AppState>) -> Result<bool, String> {
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
fn add_leave_log(log: LeaveLog, state: State<AppState>) -> Result<LeaveLog, String> {
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
fn get_leave_logs(user_id: i64, state: State<AppState>) -> Result<Vec<LeaveLog>, String> {
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
fn get_today_leave(
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
fn remove_leave_log(data: LeaveLogRequest, state: State<AppState>) -> Result<bool, String> {
    let mut conn = state
        .pool
        .get_conn()
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.exec_drop("DELETE FROM leave_logs WHERE id = ?", (data.id,))
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(true)
}

#[tauri::command]
fn get_calendar_data(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database_url = "mysql://root:@localhost:3306/boredapp";

            let state = AppState::new(database_url).expect("Failed to initialize MySQL database");

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            register,
            logout,
            get_user_by_id,
            save_attendance_record,
            get_attendance_record,
            get_attendance_records,
            get_office_hours,
            save_office_hours,
            add_leave_log,
            get_leave_logs,
            remove_leave_log,
            get_today_leave,
            get_calendar_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
