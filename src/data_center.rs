#[derive(Copy, Clone, Debug)]
pub enum DataCenter {
    Europe,
    UnitedStates,
    #[cfg(test)]
    Test,
}

impl DataCenter {
    pub fn base_url(&self) -> &str {
        match self {
            Self::Europe => "https://eapi.pcloud.com",
            Self::UnitedStates => "https://api.pcloud.com",
            #[cfg(test)]
            #[allow(deprecated)]
            Self::Test => mockito::SERVER_URL,
        }
    }
}

impl Default for DataCenter {
    fn default() -> Self {
        Self::Europe
    }
}

impl DataCenter {
    pub fn from_env() -> Self {
        if let Ok(value) = std::env::var("PCLOUD_DATA_CENTER") {
            match value.as_str() {
                "eu" => Self::Europe,
                "us" => Self::UnitedStates,
                _ => Self::default(),
            }
        } else {
            Self::default()
        }
    }
}
