use rand::Rng;

#[derive(Debug)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn from_str(token: &str) -> Option<SessionToken> {
        if !Self::is_valid_token(token) {
            return None;
        }

        Some(Self(token.to_string()))
    }

    pub fn generate_random() -> SessionToken {
        let mut rng = rand::thread_rng();

        let mut bytes: [u8; 16] = [0; 16];
        bytes.iter_mut().for_each(|b| *b = rng.gen());

        let token = hex::encode(bytes);
        debug_assert!(Self::is_valid_token(&token));

        Self(token)
    }

    fn is_valid_token(token: &str) -> bool {
        // All tokens are of length 32.
        token.len() == 32
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
