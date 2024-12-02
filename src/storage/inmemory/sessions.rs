use std::{collections::HashMap, sync::Mutex};

use crate::{app::repositories::SessionsRepository, model::{SessionToken, UserId}};

pub struct InMemorySessions {
    sessions: Mutex<HashMap<String, UserId>>,
}

impl InMemorySessions {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionsRepository for InMemorySessions {
    async fn get_authorized_user_id(&self, token: &SessionToken) -> anyhow::Result<Option<UserId>> {
        Ok(self.sessions.lock().unwrap().get(token.as_str()).copied())
    }

    async fn create_user_session(&self, user_id: UserId) -> anyhow::Result<SessionToken> {
        let r = SessionToken::generate_random();

        let mut s = self.sessions.lock().unwrap();

        if s.contains_key(r.as_str()) {
            Err(anyhow::anyhow!("could not create a unique session token"))
        } else {
            s.insert(r.as_str().to_string(), user_id);
            Ok(r)
        }
    }
}
