use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
    #[allow(dead_code)]
    pub remember_me: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserData {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub calendar_preference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarPreferencePayload {
    pub user_id: i64,
    pub calendar_preference: String, // "ad" or "bs"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub user_id: i64,
    pub date: String, 
    pub attendance_date_ad: String,
    pub attendance_date_bs: Option<String>,
    pub check_in: Option<String>,  // HH:mm
    pub check_out: Option<String>, // HH:mm
    pub status: String,
    pub overtime: i32,
    pub is_manual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeHour {
    pub day_of_week: i32,           
    pub start_time: Option<String>, // HH:mm
    pub end_time: Option<String>,   // HH:mm
    pub is_off_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveLog {
    pub id: Option<i64>,
    pub user_id: i64,
    pub leave_date: String,
    pub leave_date_ad: String,
    pub leave_date_bs: Option<String>,
    pub leave_type: String,
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
