use crate::db::AppState;
use crate::models::{CalendarPreferencePayload, UserData};
use crate::repositories::user_repo;

trait UserRepository {
    fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserData>, String>;
    fn get_calendar_preference(&self, user_id: i64) -> Result<Option<String>, String>;
    fn save_calendar_preference(&self, user_id: i64, calendar_preference: &str) -> Result<(), String>;
}

struct MySqlUserRepository<'a> {
    state: &'a AppState,
}

impl<'a> UserRepository for MySqlUserRepository<'a> {
    fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserData>, String> {
        user_repo::get_user_by_id(self.state, user_id).map_err(|e| format!("Database error: {}", e))
    }

    fn get_calendar_preference(&self, user_id: i64) -> Result<Option<String>, String> {
        user_repo::get_calendar_preference(self.state, user_id)
            .map_err(|e| format!("Database error: {}", e))
    }

    fn save_calendar_preference(&self, user_id: i64, calendar_preference: &str) -> Result<(), String> {
        user_repo::save_calendar_preference(self.state, user_id, calendar_preference)
            .map_err(|e| format!("Database error: {}", e))
    }
}

pub fn get_user_by_id(state: &AppState, user_id: i64) -> Option<UserData> {
    let repo = MySqlUserRepository { state };
    get_user_by_id_with_repo(&repo, user_id)
}

pub fn get_calendar_preference(state: &AppState, user_id: i64) -> Result<String, String> {
    let repo = MySqlUserRepository { state };
    get_calendar_preference_with_repo(&repo, user_id)
}

pub fn save_calendar_preference(
    state: &AppState,
    payload: CalendarPreferencePayload,
) -> Result<bool, String> {
    let repo = MySqlUserRepository { state };
    save_calendar_preference_with_repo(&repo, payload)
}

fn get_user_by_id_with_repo(repo: &dyn UserRepository, user_id: i64) -> Option<UserData> {
    repo.get_user_by_id(user_id).ok().flatten()
}

fn get_calendar_preference_with_repo(repo: &dyn UserRepository, user_id: i64) -> Result<String, String> {
    let result = repo.get_calendar_preference(user_id)?;

    Ok(result.unwrap_or_else(|| "ad".to_string()))
}

fn save_calendar_preference_with_repo(
    repo: &dyn UserRepository,
    payload: CalendarPreferencePayload,
) -> Result<bool, String> {
    let value = payload.calendar_preference.to_lowercase();
    if value != "ad" && value != "bs" {
        return Err("Invalid calendar preference. Use 'ad' or 'bs'.".to_string());
    }

    repo.save_calendar_preference(payload.user_id, &value)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserRepository {
        user: Option<UserData>,
        calendar_preference: Option<String>,
        fail_get: bool,
        fail_save: bool,
    }

    impl UserRepository for MockUserRepository {
        fn get_user_by_id(&self, _user_id: i64) -> Result<Option<UserData>, String> {
            if self.fail_get {
                return Err("get failed".to_string());
            }
            Ok(self.user.clone())
        }

        fn get_calendar_preference(&self, _user_id: i64) -> Result<Option<String>, String> {
            if self.fail_get {
                return Err("get failed".to_string());
            }
            Ok(self.calendar_preference.clone())
        }

        fn save_calendar_preference(&self, _user_id: i64, _calendar_preference: &str) -> Result<(), String> {
            if self.fail_save {
                return Err("save failed".to_string());
            }
            Ok(())
        }
    }

    #[test]
    fn get_calendar_preference_defaults_to_ad() {
        let repo = MockUserRepository {
            user: None,
            calendar_preference: None,
            fail_get: false,
            fail_save: false,
        };

        let result = get_calendar_preference_with_repo(&repo, 1).expect("should succeed");
        assert_eq!(result, "ad");
    }

    #[test]
    fn save_calendar_preference_rejects_invalid_value() {
        let repo = MockUserRepository {
            user: None,
            calendar_preference: Some("ad".to_string()),
            fail_get: false,
            fail_save: false,
        };

        let result = save_calendar_preference_with_repo(
            &repo,
            CalendarPreferencePayload {
                user_id: 1,
                calendar_preference: "invalid".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn get_user_by_id_returns_none_on_repo_failure() {
        let repo = MockUserRepository {
            user: Some(UserData {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                calendar_preference: "ad".to_string(),
            }),
            calendar_preference: Some("ad".to_string()),
            fail_get: true,
            fail_save: false,
        };

        let user = get_user_by_id_with_repo(&repo, 1);
        assert!(user.is_none());
    }
}
