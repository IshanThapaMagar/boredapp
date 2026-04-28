use crate::db::AppState;
use crate::models::*;
use crate::services::{
    attendance_service, auth_service, calendar_service, leave_service, office_service,
    user_service,
};
use tauri::State;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You're ready to get started with the app!", name)
}

#[tauri::command]
pub fn login(login_data: LoginRequest, state: State<AppState>) -> LoginResponse {
    auth_service::login(state.inner(), login_data)
}

#[tauri::command]
pub fn register(user: User, state: State<AppState>) -> LoginResponse {
    auth_service::register(state.inner(), user)
}

#[tauri::command]
pub fn logout() -> bool {
    true
}

#[tauri::command]
pub fn get_user_by_id(user_id: i64, state: State<AppState>) -> Option<UserData> {
    user_service::get_user_by_id(state.inner(), user_id)
}

#[tauri::command]
pub fn get_calendar_preference(user_id: i64, state: State<AppState>) -> Result<String, String> {
    user_service::get_calendar_preference(state.inner(), user_id)
}

#[tauri::command]
pub fn save_calendar_preference(
    payload: CalendarPreferencePayload,
    state: State<AppState>,
) -> Result<bool, String> {
    user_service::save_calendar_preference(state.inner(), payload)
}

#[tauri::command]
pub fn save_attendance_record(
    record: AttendanceRecord,
    state: State<AppState>,
) -> Result<bool, String> {
    attendance_service::save_attendance_record(state.inner(), record)
}

#[tauri::command]
pub fn get_attendance_record(
    user_id: i64,
    date: String,
    state: State<AppState>,
) -> Result<Option<AttendanceRecord>, String> {
    attendance_service::get_attendance_record(state.inner(), user_id, date)
}

#[tauri::command]
pub fn get_attendance_records(
    user_id: i64,
    start_date: String,
    end_date: String,
    state: State<AppState>,
) -> Result<Vec<AttendanceRecord>, String> {
    attendance_service::get_attendance_records(state.inner(), user_id, start_date, end_date)
}

#[tauri::command]
pub fn get_office_hours(user_id: i64, state: State<AppState>) -> Result<Vec<OfficeHour>, String> {
    office_service::get_office_hours(state.inner(), user_id)
}

#[tauri::command]
pub fn save_office_hours(
    user_id: i64,
    hours: Vec<OfficeHour>,
    state: State<AppState>,
) -> Result<bool, String> {
    office_service::save_office_hours(state.inner(), user_id, hours)
}

#[tauri::command]
pub fn add_leave_log(log: LeaveLog, state: State<AppState>) -> Result<LeaveLog, String> {
    leave_service::add_leave_log(state.inner(), log)
}

#[tauri::command]
pub fn get_leave_logs(user_id: i64, state: State<AppState>) -> Result<Vec<LeaveLog>, String> {
    leave_service::get_leave_logs(state.inner(), user_id)
}

#[tauri::command]
pub fn get_today_leave(
    user_id: i64,
    date: String,
    state: State<AppState>,
) -> Result<Option<LeaveLog>, String> {
    leave_service::get_today_leave(state.inner(), user_id, date)
}

#[tauri::command]
pub fn remove_leave_log(data: LeaveLogRequest, state: State<AppState>) -> Result<bool, String> {
    leave_service::remove_leave_log(state.inner(), data)
}

#[tauri::command]
pub fn get_calendar_data(
    start_date: String,
    end_date: String,
    state: State<AppState>,
) -> Result<Vec<CalendarDay>, String> {
    calendar_service::get_calendar_data(state.inner(), start_date, end_date)
}
