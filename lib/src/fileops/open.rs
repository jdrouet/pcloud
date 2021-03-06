use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::file::FileIdentifier;
use crate::prelude::BinaryCommand;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    fd: u64,
}

#[derive(Debug)]
pub struct FileOpenCommand {
    flags: u16,
    identifier: Option<FileIdentifier>,
    folder_id: Option<u64>,
    name: Option<String>,
}

impl FileOpenCommand {
    pub fn new(flags: u16) -> Self {
        Self {
            flags,
            identifier: None,
            folder_id: None,
            name: None,
        }
    }

    pub fn identifier(mut self, value: FileIdentifier) -> Self {
        self.identifier = Some(value);
        self
    }

    pub fn folder_id(mut self, value: u64) -> Self {
        self.folder_id = Some(value);
        self
    }

    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        let mut res = vec![("flags", BinaryValue::Number(self.flags as u64))];
        if let Some(ref identifier) = self.identifier {
            res.extend_from_slice(&identifier.to_binary_params());
        }
        if let Some(folder_id) = self.folder_id {
            res.push(("folderid", BinaryValue::Number(folder_id)));
        }
        if let Some(ref name) = self.name {
            res.push(("name", BinaryValue::Text(name.to_string())));
        }
        res
    }
}

impl BinaryCommand for FileOpenCommand {
    type Output = u64;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let res = client.send_command("file_open", &self.to_binary_params())?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload().map(|value| value.fd)
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::FileOpenCommand;
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::prelude::BinaryCommand;
    use crate::region::Region;

    #[test]
    fn open_existing_file() {
        let creds = Credentials::from_env().unwrap();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        FileOpenCommand::new(0)
            .identifier(5837100991.into())
            .execute(&mut client)
            .unwrap();
    }
}
