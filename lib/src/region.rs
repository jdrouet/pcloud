#[derive(Copy, Clone, Debug)]
pub enum Region {
    Europe,
    UnitedStates,
    #[cfg(test)]
    Test,
}

impl Region {
    pub fn base_url(&self) -> &str {
        match self {
            Self::Europe => "https://eapi.pcloud.com",
            Self::UnitedStates => "https://api.pcloud.com",
            #[cfg(test)]
            #[allow(deprecated)]
            Self::Test => mockito::SERVER_URL,
        }
    }

    pub fn address(&self) -> &str {
        match self {
            Self::Europe => "eapi.pcloud.com",
            Self::UnitedStates => "api.pcloud.com",
            #[cfg(test)]
            #[allow(deprecated)]
            Self::Test => mockito::SERVER_URL,
        }
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::Europe
    }
}

impl Region {
    pub fn from_env() -> Self {
        if let Ok(value) = std::env::var("PCLOUD_REGION") {
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
