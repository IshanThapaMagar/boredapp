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
                calendar_preference VARCHAR(2) NOT NULL DEFAULT 'ad',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )?;

        // Migration: add users.calendar_preference if missing
        let calendar_pref_column_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'users' AND COLUMN_NAME = 'calendar_preference' AND TABLE_SCHEMA = DATABASE()"
        )?;

        if calendar_pref_column_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE users ADD COLUMN calendar_preference VARCHAR(2) NOT NULL DEFAULT 'ad' AFTER password_hash",
            )?;
        }

        // Create attendance table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS attendance (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                attendance_date DATE NOT NULL,
                attendance_date_ad DATE,
                attendance_date_bs VARCHAR(20),
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

        let attendance_ad_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'attendance' AND COLUMN_NAME = 'attendance_date_ad' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if attendance_ad_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE attendance ADD COLUMN attendance_date_ad DATE NULL AFTER attendance_date",
            )?;
        }

        let attendance_bs_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'attendance' AND COLUMN_NAME = 'attendance_date_bs' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if attendance_bs_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE attendance ADD COLUMN attendance_date_bs VARCHAR(20) NULL AFTER attendance_date_ad",
            )?;
        }

        conn.query_drop(
            "UPDATE attendance SET attendance_date_ad = attendance_date WHERE attendance_date_ad IS NULL",
        )?;

        let attendance_unique_key_exists: Option<String> = conn.query_first(
            "SELECT INDEX_NAME FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_NAME = 'attendance' AND INDEX_NAME = 'user_date_ad' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if attendance_unique_key_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE attendance ADD UNIQUE KEY user_date_ad (user_id, attendance_date_ad)",
            )?;
        }

        // Create per-user office_working_hours table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS office_working_hours (
                user_id BIGINT NOT NULL,
                day_of_week INT NOT NULL,
                start_time VARCHAR(8),
                end_time VARCHAR(8),
                is_off_day BOOLEAN DEFAULT FALSE,
                PRIMARY KEY (user_id, day_of_week),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )?;

        // Migration support for old office_working_hours schema without user_id
        let office_user_column_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'office_working_hours' AND COLUMN_NAME = 'user_id' AND TABLE_SCHEMA = DATABASE()"
        )?;

        if office_user_column_exists.is_none() {
            conn.query_drop(
                "DROP TABLE office_working_hours",
            )?;

            conn.query_drop(
                r#"
                CREATE TABLE IF NOT EXISTS office_working_hours (
                    user_id BIGINT NOT NULL,
                    day_of_week INT NOT NULL,
                    start_time VARCHAR(8),
                    end_time VARCHAR(8),
                    is_off_day BOOLEAN DEFAULT FALSE,
                    PRIMARY KEY (user_id, day_of_week),
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
                )
                "#,
            )?;
        }

        // Create leave_logs table
        conn.query_drop(
            r#"
            CREATE TABLE IF NOT EXISTS leave_logs (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                leave_date DATE NOT NULL,
                leave_date_ad DATE,
                leave_date_bs VARCHAR(20),
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

        let leave_ad_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'leave_logs' AND COLUMN_NAME = 'leave_date_ad' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if leave_ad_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE leave_logs ADD COLUMN leave_date_ad DATE NULL AFTER leave_date",
            )?;
        }

        let leave_bs_exists: Option<String> = conn.query_first(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'leave_logs' AND COLUMN_NAME = 'leave_date_bs' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if leave_bs_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE leave_logs ADD COLUMN leave_date_bs VARCHAR(20) NULL AFTER leave_date_ad",
            )?;
        }

        conn.query_drop(
            "UPDATE leave_logs SET leave_date_ad = leave_date WHERE leave_date_ad IS NULL",
        )?;
        conn.query_drop(
            "UPDATE leave_logs SET leave_date_bs = absent_date_bs WHERE leave_date_bs IS NULL AND absent_date_bs IS NOT NULL",
        )?;

        let leave_unique_key_exists: Option<String> = conn.query_first(
            "SELECT INDEX_NAME FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_NAME = 'leave_logs' AND INDEX_NAME = 'user_leave_date_ad' AND TABLE_SCHEMA = DATABASE()"
        )?;
        if leave_unique_key_exists.is_none() {
            conn.query_drop(
                "ALTER TABLE leave_logs ADD UNIQUE KEY user_leave_date_ad (user_id, leave_date_ad)",
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

        // Ensure every user has default office hours rows.
        let user_ids: Vec<i64> = conn.query("SELECT id FROM users")?;
        for user_id in user_ids {
            let office_count: Option<u64> = conn.exec_first(
                "SELECT COUNT(*) FROM office_working_hours WHERE user_id = ?",
                (user_id,),
            )?;

            if office_count.unwrap_or(0) == 0 {
                for day in 0..=6 {
                    let (start_time, end_time, is_off_day) = if day == 6 {
                        (None, None, true)
                    } else {
                        (Some("09:00".to_string()), Some("17:00".to_string()), false)
                    };

                    conn.exec_drop(
                        "INSERT INTO office_working_hours (user_id, day_of_week, start_time, end_time, is_off_day) VALUES (?, ?, ?, ?, ?)",
                        (user_id, day, start_time, end_time, is_off_day),
                    )?;
                }
            }
        }

        Ok(Self { pool })
    }
}
