use crate::db::AppState;
use crate::models::{LoginRequest, LoginResponse, User, UserData};
use crate::repositories::auth_repo;

trait AuthRepository {
    fn find_user_for_login(
        &self,
        email_or_username: &str,
    ) -> Result<Option<(i64, String, String, String, String)>, String>;

    fn find_existing_user_by_email_or_username(
        &self,
        email: &str,
        username: &str,
    ) -> Result<Option<i64>, String>;

    fn insert_user(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, String>;
}

struct MySqlAuthRepository<'a> {
    state: &'a AppState,
}

impl<'a> AuthRepository for MySqlAuthRepository<'a> {
    fn find_user_for_login(
        &self,
        email_or_username: &str,
    ) -> Result<Option<(i64, String, String, String, String)>, String> {
        auth_repo::find_user_for_login(self.state, email_or_username)
            .map_err(|e| format!("Database error: {}", e))
    }

    fn find_existing_user_by_email_or_username(
        &self,
        email: &str,
        username: &str,
    ) -> Result<Option<i64>, String> {
        auth_repo::find_existing_user_by_email_or_username(self.state, email, username)
            .map_err(|e| format!("Database error: {}", e))
    }

    fn insert_user(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, String> {
        auth_repo::insert_user(self.state, username, email, password_hash)
            .map_err(|e| format!("Database error: {}", e))
    }
}

pub fn login(state: &AppState, login_data: LoginRequest) -> LoginResponse {
    let repo = MySqlAuthRepository { state };
    login_with_repo(&repo, login_data)
}

fn login_with_repo(repo: &dyn AuthRepository, login_data: LoginRequest) -> LoginResponse {
    let result = repo.find_user_for_login(&login_data.email_or_username);

    match result {
        Ok(Some((id, username, email, password_hash, calendar_preference))) => {
            if bcrypt::verify(&login_data.password, &password_hash).unwrap_or(false) {
                LoginResponse {
                    success: true,
                    message: "Login successful!".into(),
                    user: Some(UserData {
                        id,
                        username,
                        email,
                        calendar_preference,
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
        _ => LoginResponse {
            success: false,
            message: "Invalid email/username or password".into(),
            user: None,
        },
    }
}

pub fn register(state: &AppState, user: User) -> LoginResponse {
    let repo = MySqlAuthRepository { state };
    register_with_repo(&repo, user)
}

fn register_with_repo(repo: &dyn AuthRepository, user: User) -> LoginResponse {
    match repo.find_existing_user_by_email_or_username(&user.email, &user.username) {
        Ok(Some(_)) => {
            return LoginResponse {
                success: false,
                message: "User with this email or username already exists".into(),
                user: None,
            }
        }
        Ok(None) => {}
        Err(_) => {
            return LoginResponse {
                success: false,
                message: "Error processing registration".into(),
                user: None,
            }
        }
    }

    let password_hash = match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return LoginResponse {
                success: false,
                message: "Error processing registration".into(),
                user: None,
            }
        }
    };

    let user_id = match repo.insert_user(&user.username, &user.email, &password_hash) {
        Ok(id) => id,
        Err(_) => {
            return LoginResponse {
                success: false,
                message: "Error processing registration".into(),
                user: None,
            }
        }
    };

    LoginResponse {
        success: true,
        message: "Registration successful!".into(),
        user: Some(UserData {
            id: user_id,
            username: user.username,
            email: user.email,
            calendar_preference: "ad".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAuthRepository {
        user_for_login: Option<(i64, String, String, String, String)>,
        existing_user_id: Option<i64>,
        next_inserted_id: i64,
        fail_read: bool,
        fail_insert: bool,
    }

    impl AuthRepository for MockAuthRepository {
        fn find_user_for_login(
            &self,
            _email_or_username: &str,
        ) -> Result<Option<(i64, String, String, String, String)>, String> {
            if self.fail_read {
                return Err("read failed".to_string());
            }
            Ok(self.user_for_login.clone())
        }

        fn find_existing_user_by_email_or_username(
            &self,
            _email: &str,
            _username: &str,
        ) -> Result<Option<i64>, String> {
            if self.fail_read {
                return Err("read failed".to_string());
            }
            Ok(self.existing_user_id)
        }

        fn insert_user(&self, _username: &str, _email: &str, _password_hash: &str) -> Result<i64, String> {
            if self.fail_insert {
                return Err("insert failed".to_string());
            }
            Ok(self.next_inserted_id)
        }
    }

    #[test]
    fn login_succeeds_with_valid_password() {
        let hashed = bcrypt::hash("secret", bcrypt::DEFAULT_COST).expect("hash should succeed");
        let repo = MockAuthRepository {
            user_for_login: Some((
                1,
                "alice".to_string(),
                "alice@example.com".to_string(),
                hashed,
                "ad".to_string(),
            )),
            existing_user_id: None,
            next_inserted_id: 0,
            fail_read: false,
            fail_insert: false,
        };

        let result = login_with_repo(
            &repo,
            LoginRequest {
                email_or_username: "alice".to_string(),
                password: "secret".to_string(),
                remember_me: false,
            },
        );

        assert!(result.success);
        assert_eq!(result.message, "Login successful!");
        assert!(result.user.is_some());
    }

    #[test]
    fn login_fails_with_invalid_password() {
        let hashed = bcrypt::hash("secret", bcrypt::DEFAULT_COST).expect("hash should succeed");
        let repo = MockAuthRepository {
            user_for_login: Some((
                1,
                "alice".to_string(),
                "alice@example.com".to_string(),
                hashed,
                "ad".to_string(),
            )),
            existing_user_id: None,
            next_inserted_id: 0,
            fail_read: false,
            fail_insert: false,
        };

        let result = login_with_repo(
            &repo,
            LoginRequest {
                email_or_username: "alice".to_string(),
                password: "wrong".to_string(),
                remember_me: false,
            },
        );

        assert!(!result.success);
        assert!(result.user.is_none());
    }

    #[test]
    fn register_fails_when_user_exists() {
        let repo = MockAuthRepository {
            user_for_login: None,
            existing_user_id: Some(42),
            next_inserted_id: 100,
            fail_read: false,
            fail_insert: false,
        };

        let result = register_with_repo(
            &repo,
            User {
                id: None,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                password: "secret".to_string(),
            },
        );

        assert!(!result.success);
        assert_eq!(result.message, "User with this email or username already exists");
    }

    #[test]
    fn register_succeeds_for_new_user() {
        let repo = MockAuthRepository {
            user_for_login: None,
            existing_user_id: None,
            next_inserted_id: 100,
            fail_read: false,
            fail_insert: false,
        };

        let result = register_with_repo(
            &repo,
            User {
                id: None,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                password: "secret".to_string(),
            },
        );

        assert!(result.success);
        assert_eq!(result.user.expect("user should exist").id, 100);
    }
}
