use mysql::prelude::Queryable;
use mysql::Pool;

pub struct AppState {
    pub pool: Pool,
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

        // Migration: Ensure attendance_date column exists
        let column_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'attendance' AND COLUMN_NAME = 'attendance_date' AND TABLE_SCHEMA = DATABASE()"
        )?;

        if column_exists.is_none() {
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
                    (None, None, true)
                } else {
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
        let user_count: Option<u64> = conn.query_first("SELECT COUNT(*) FROM users")?;
        if user_count.unwrap_or(0) == 0 {
            let admin_hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap();
            conn.exec_drop(
                "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
                ("admin", "admin@example.com", admin_hash),
            )?;
        }

        Ok(Self { pool })
    }
}
