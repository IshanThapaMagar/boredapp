mod models;
mod db;
mod commands;

use tauri::Manager;
use db::AppState;

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
            commands::greet,
            commands::login,
            commands::register,
            commands::logout,
            commands::get_user_by_id,
            commands::get_calendar_preference,
            commands::save_calendar_preference,
            commands::save_attendance_record,
            commands::get_attendance_record,
            commands::get_attendance_records,
            commands::get_office_hours,
            commands::save_office_hours,
            commands::add_leave_log,
            commands::get_leave_logs,
            commands::remove_leave_log,
            commands::get_today_leave,
            commands::get_calendar_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
