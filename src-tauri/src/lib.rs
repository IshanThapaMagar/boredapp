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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database_url = "mysql://root:root@localhost:3306/boredapp";

            let state = AppState::new(database_url).expect("Failed to initialize MySQL database");

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            register,
            logout,
            get_user_by_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
