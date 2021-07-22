use super::PCloudApi;
use crate::common::RemoteFile;
use crate::request::Error as RequestError;
use serde_json::Value as JsonValue;

pub const ROOT: usize = 0;

#[derive(Debug, serde::Deserialize)]
pub struct Response {
    result: u16,
    metadata: Option<RemoteFile>,
    #[serde(flatten)]
    extra: JsonValue,
}

impl Response {
    fn metadata(self) -> Result<RemoteFile, RequestError> {
        if let Some(content) = self.metadata {
            Ok(content)
        } else {
            Err(RequestError::Payload(self.result))
        }
    }
}

impl PCloudApi {
    pub async fn create_folder(
        &self,
        name: &str,
        parent_id: usize,
    ) -> Result<RemoteFile, RequestError> {
        let parent_id = parent_id.to_string();
        let params = vec![("name", name), ("folderid", parent_id.as_str())];
        let result: Response = self.get_request("createfolder", &params).await?;
        result.metadata()
    }
}

#[derive(Debug, Default)]
pub struct ListFolderParams {
    recursive: bool,
    show_deleted: bool,
    no_files: bool,
    no_shares: bool,
}

impl ListFolderParams {
    /// If is set full directory tree will be returned, which means that all directories will have contents filed.
    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }

    /// If is set, deleted files and folders that can be undeleted will be displayed.
    pub fn show_deleted(mut self, value: bool) -> Self {
        self.show_deleted = value;
        self
    }

    /// If is set, only the folder (sub)structure will be returned.
    pub fn no_files(mut self, value: bool) -> Self {
        self.no_files = value;
        self
    }

    /// If is set, only user's own folders and files will be displayed.
    pub fn no_shares(mut self, value: bool) -> Self {
        self.no_shares = value;
        self
    }

    fn to_vec(&self) -> Vec<(&str, &str)> {
        let mut res = vec![];
        if self.recursive {
            res.push(("recursive", "1"));
        }
        if self.show_deleted {
            res.push(("showdeleted", "1"));
        }
        if self.no_files {
            res.push(("no_files", "1"));
        }
        if self.no_shares {
            res.push(("no_shares", "1"));
        }
        res
    }
}

impl PCloudApi {
    pub async fn list_folder(&self, folder_id: usize) -> Result<RemoteFile, RequestError> {
        self.list_folder_with_params(folder_id, &ListFolderParams::default())
            .await
    }

    pub async fn list_folder_with_params(
        &self,
        folder_id: usize,
        params: &ListFolderParams,
    ) -> Result<RemoteFile, RequestError> {
        let folder_id = folder_id.to_string();
        let mut local_params = vec![("folderid", folder_id.as_str())];
        local_params.extend(&params.to_vec());
        let result: Response = self.get_request("listfolder", &local_params).await?;
        result.metadata()
    }
}

impl PCloudApi {
    pub async fn delete_folder(&self, folder_id: usize) -> Result<RemoteFile, RequestError> {
        let folder_id = folder_id.to_string();
        let params = vec![("folderid", folder_id.as_str())];
        let result: Response = self.get_request("deletefolder", &params).await?;
        result.metadata()
    }
}

#[derive(Debug, serde::Deserialize)]
struct ResponseDeleteRecursive {
    result: u16,
    deletedfiles: Option<usize>,
    deletedfolders: Option<usize>,
}

impl ResponseDeleteRecursive {
    fn metadata(self) -> Result<(usize, usize), RequestError> {
        if let (Some(deletedfiles), Some(deletedfolders)) = (self.deletedfiles, self.deletedfolders)
        {
            Ok((deletedfiles, deletedfolders))
        } else {
            Err(RequestError::Payload(self.result))
        }
    }
}

impl PCloudApi {
    pub async fn delete_folder_recursive(
        &self,
        folder_id: usize,
    ) -> Result<(usize, usize), RequestError> {
        let folder_id = folder_id.to_string();
        let params = vec![("folderid", folder_id.as_str())];
        let result: ResponseDeleteRecursive =
            self.get_request("deletefolderrecursive", &params).await?;
        result.metadata()
    }
}

impl PCloudApi {
    pub async fn rename_folder(
        &self,
        folder_id: usize,
        name: &str,
    ) -> Result<RemoteFile, RequestError> {
        let folder_id = folder_id.to_string();
        let params = vec![("folderid", folder_id.as_str()), ("toname", name)];
        let result: Response = self.get_request("renamefolder", &params).await?;
        result.metadata()
    }

    pub async fn move_folder(
        &self,
        folder_id: usize,
        to_folder_id: usize,
    ) -> Result<RemoteFile, RequestError> {
        let folder_id = folder_id.to_string();
        let to_folder_id = to_folder_id.to_string();
        let params = vec![
            ("folderid", folder_id.as_str()),
            ("tofolderid", to_folder_id.as_str()),
        ];
        let result: Response = self.get_request("renamefolder", &params).await?;
        result.metadata()
    }
}
