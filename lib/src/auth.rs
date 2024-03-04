#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Credential {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: u32,
    id_token: String,
    #[serde(skip, default = "String::new")]
    _refresh_token: String,
}

impl Credential {
    pub fn get_id_token(&self) -> &str {
        &self.id_token
    }

    pub fn _get_access_token(&self) -> &str {
        &self.access_token
    }

    pub fn _get_refresh_token(&self) -> &str {
        &self._refresh_token
    }
}

impl TryFrom<&std::path::PathBuf> for Credential {
    type Error = String;

    fn try_from(path: &std::path::PathBuf) -> Result<Self, Self::Error> {
        let s = std::fs::read_to_string(path)
            .map_err(|_| "Failed to read credential file".to_string())?;

        s.as_str().try_into()
    }
}

impl TryFrom<&str> for Credential {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(s).map_err(|_| "Failed to parse credential string".to_string())
    }
}
