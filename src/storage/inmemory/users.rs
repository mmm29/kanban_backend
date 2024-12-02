use std::{collections::HashMap, sync::Mutex};

use crate::{app::repositories::UsersRepositry, model::UserId};

struct UserStorage {
    username: String,
    password: String,
}

struct MutableUsersStorage {
    next_id: UserId,
    users_by_id: HashMap<UserId, UserStorage>,
    users_by_name: HashMap<String, UserId>,
}

pub struct InMemoryUsers {
    users: Mutex<MutableUsersStorage>,
}

impl InMemoryUsers {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(MutableUsersStorage {
                next_id: UserId::from_raw(1),
                users_by_id: HashMap::new(),
                users_by_name: HashMap::new(),
            }),
        }
    }

    pub fn add_user(&self, user_id: UserId, username: &str, password: &str) -> anyhow::Result<()> {
        let mut users = self.users.lock().unwrap();

        users.next_id = UserId::from_raw(user_id.raw() + 1);

        users.users_by_id.insert(
            user_id,
            UserStorage {
                username: username.to_string(),
                password: password.to_string(),
            },
        );

        users.users_by_name.insert(username.to_string(), user_id);
        Ok(())
    }
}

#[async_trait]
impl UsersRepositry for InMemoryUsers {
    async fn does_user_exist_by_username(&self, username: &str) -> anyhow::Result<bool> {
        let users = self.users.lock().unwrap();
        Ok(users.users_by_name.contains_key(username))
    }

    async fn get_username(&self, user_id: UserId) -> anyhow::Result<Option<String>> {
        let users = self.users.lock().unwrap();

        Ok(users.users_by_id.get(&user_id).map(|x| x.username.clone()))
    }

    async fn create_user(&self, username: &str, password: &str) -> anyhow::Result<UserId> {
        let mut users = self.users.lock().unwrap();

        let user_id = users.next_id;
        users.next_id = UserId::from_raw(user_id.raw() + 1);

        users.users_by_id.insert(
            user_id,
            UserStorage {
                username: username.to_string(),
                password: password.to_string(),
            },
        );
        users.users_by_name.insert(username.to_string(), user_id);

        Ok(user_id)
    }

    async fn find_user_with_password(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<(UserId, String)>> {
        let users = self.users.lock().unwrap();

        let Some(user_id) = users.users_by_name.get(username).copied() else {
            return Ok(None);
        };

        // The user id is guaranteed to exist
        let user = users.users_by_id.get(&user_id).unwrap();

        Ok(Some((user_id, user.password.clone())))
    }
}
