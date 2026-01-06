use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};
use rusqlite::{Connection, Result as SqlResult};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::path::PathBuf;

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
    db: Mutex<Connection>,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        
        // Create users table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Add demo users if table is empty
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0)
        )?;

        if count == 0 {
            let admin_hash = hash("admin", DEFAULT_COST).unwrap();

            conn.execute(
                "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
                ["admin", "admin@example.com", &admin_hash],
            )?;
        }

        Ok(AppState {
            db: Mutex::new(conn),
        })
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You're ready to get started with the app!", name)
}

#[tauri::command]
fn login(login_data: LoginRequest, state: State<AppState>) -> LoginResponse {
    let db = state.db.lock().unwrap();

    let result = db.query_row(
        "SELECT id, username, email, password_hash FROM users 
         WHERE email = ?1 OR username = ?1",
        [&login_data.email_or_username],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    match result {
        Ok((id, username, email, password_hash)) => {
            // Verify password
            match verify(&login_data.password, &password_hash) {
                Ok(true) => LoginResponse {
                    success: true,
                    message: String::from("Login successful!"),
                    user: Some(UserData {
                        id,
                        username,
                        email,
                    }),
                },
                _ => LoginResponse {
                    success: false,
                    message: String::from("Invalid email/username or password"),
                    user: None,
                },
            }
        }
        Err(_) => LoginResponse {
            success: false,
            message: String::from("Invalid email/username or password"),
            user: None,
        },
    }
}

#[tauri::command]
fn register(user: User, state: State<AppState>) -> LoginResponse {
    let db = state.db.lock().unwrap();


    let exists: Result<i64, _> = db.query_row(
        "SELECT id FROM users WHERE email = ?1 OR username = ?2",
        [&user.email, &user.username],
        |row| row.get(0),
    );

    if exists.is_ok() {
        return LoginResponse {
            success: false,
            message: String::from("User with this email or username already exists"),
            user: None,
        };
    }

    // Hash password
    let password_hash = match hash(&user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return LoginResponse {
                success: false,
                message: String::from("Error processing registration"),
                user: None,
            }
        }
    };

    // Insert new user
    match db.execute(
        "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
        [&user.username, &user.email, &password_hash],
    ) {
        Ok(_) => {
            let user_id = db.last_insert_rowid();
            LoginResponse {
                success: true,
                message: String::from("Registration successful!"),
                user: Some(UserData {
                    id: user_id,
                    username: user.username,
                    email: user.email,
                }),
            }
        }
        Err(e) => LoginResponse {
            success: false,
            message: format!("Registration failed: {}", e),
            user: None,
        },
    }
}

#[tauri::command]
fn logout() -> bool {
    // Clear session/token in production
    true
}

#[tauri::command]
fn get_user_by_id(user_id: i64, state: State<AppState>) -> Option<UserData> {
    let db = state.db.lock().unwrap();
    
    db.query_row(
        "SELECT id, username, email FROM users WHERE id = ?1",
        [user_id],
        |row| {
            Ok(UserData {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
            })
        },
    ).ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_data_dir).unwrap();
            let db_path = app_data_dir.join("app_database.db");
            
            let state = AppState::new(db_path).expect("Failed to initialize database");
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