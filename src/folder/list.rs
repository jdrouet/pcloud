use super::FolderResponse;
use crate::common::RemoteFile;
use crate::request::{Error, Response};
use crate::PCloudApi;

#[derive(Debug, Default)]
pub struct Params {
    recursive: bool,
    show_deleted: bool,
    no_files: bool,
    no_shares: bool,
}

impl Params {
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
    /// List a folder's content
    ///
    /// # Arguments
    ///
    /// * `folder_id` - ID of the folder.
    ///
    pub async fn list_folder(&self, folder_id: usize) -> Result<RemoteFile, Error> {
        self.list_folder_with_params(folder_id, &Params::default())
            .await
    }

    pub async fn list_folder_with_params(
        &self,
        folder_id: usize,
        params: &Params,
    ) -> Result<RemoteFile, Error> {
        let folder_id = folder_id.to_string();
        let mut local_params = vec![("folderid", folder_id.as_str())];
        local_params.extend(&params.to_vec());
        let result: Response<FolderResponse> =
            self.get_request("listfolder", &local_params).await?;
        result.payload().map(|item| item.metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::data_center::DataCenter;
    use crate::PCloudApi;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "path": "\/testing",
        "name": "testing",
        "created": "Fri, 23 Jul 2021 19:39:09 +0000",
        "ismine": true,
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
        "id": "d10",
        "isshared": false,
        "icon": "folder",
        "isfolder": true,
        "parentfolderid": 0,
        "folderid": 10
    }
}"#,
            )
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = DataCenter::Test;
        let api = PCloudApi::new(creds, dc);
        let payload = api.list_folder(0).await.unwrap();
        assert_eq!(payload.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        crate::tests::init();
        let m = mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = DataCenter::Test;
        let api = PCloudApi::new(creds, dc);
        let error = api.list_folder(0).await.unwrap_err();
        assert!(matches!(error, crate::request::Error::Payload(_, _)));
        m.assert();
    }
}
