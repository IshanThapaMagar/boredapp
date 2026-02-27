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
                "ALTER TABLE attendance ADD UNIQUE KEY user_date (user_id, attendance_date)",
            )?;
        }

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
            get_attendance_record
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
