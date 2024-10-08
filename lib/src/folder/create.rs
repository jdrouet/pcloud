//! Resources needed to copy a folder

use std::borrow::Cow;

/// Command to create a folder in a defined folder
///
/// Executing this command will return a [`Folder`](crate::entry::Folder) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/folder/create.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::folder::create::FolderCreateCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FolderCreateCommand::new("foo".to_string(), 42);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FolderCreateCommand<'a> {
    pub name: Cow<'a, str>,
    pub parent_id: u64,
    pub ignore_exists: bool,
}

impl<'a> FolderCreateCommand<'a> {
    pub fn new<N: Into<Cow<'a, str>>>(name: N, parent_id: u64) -> Self {
        Self {
            name: name.into(),
            parent_id,
            ignore_exists: false,
        }
    }

    pub fn set_ignore_exists(&mut self, value: bool) {
        self.ignore_exists = value;
    }

    pub fn with_ignore_exists(mut self, value: bool) -> Self {
        self.ignore_exists = value;
        self
    }

    #[cfg(feature = "client-http")]
    fn method(&self) -> &'static str {
        if self.ignore_exists {
            "createfolderifnotexists"
        } else {
            "createfolder"
        }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use std::borrow::Cow;

    use super::FolderCreateCommand;
    use crate::client::HttpClient;
    use crate::entry::Folder;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::prelude::HttpCommand;

    #[derive(serde::Serialize)]
    struct FolderCreateParams<'a> {
        name: Cow<'a, str>,
        #[serde(rename = "folderid")]
        folder_id: u64,
    }

    impl<'a> From<FolderCreateCommand<'a>> for FolderCreateParams<'a> {
        fn from(value: FolderCreateCommand<'a>) -> Self {
            Self {
                name: value.name,
                folder_id: value.parent_id,
            }
        }
    }

    #[async_trait::async_trait]
    impl<'a> HttpCommand for FolderCreateCommand<'a> {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let method = self.method();
            let params = FolderCreateParams::from(self);
            client
                .get_request::<FolderResponse, _>(method, params)
                .await
                .map(|item| item.metadata)
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FolderCreateCommand;
    use crate::client::HttpClient;
    use crate::credentials::Credentials;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/createfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("name".into(), "testing".into()),
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
        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let result = FolderCreateCommand::new("testing", 0)
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/createfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("name".into(), "testing".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let error = FolderCreateCommand::new("testing", 0)
            .execute(&api)
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Protocol(_, _)));
        m.assert();
    }
}
