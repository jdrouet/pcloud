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
            Self::Test => "http://127.0.0.1:1234",
        }
    }
}
