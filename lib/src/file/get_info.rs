use super::FileIdentifier;
use crate::entry::Entry;
use crate::error::Error;
use crate::http::PCloudApi;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct CheckSumFile {
    pub sha256: String,
    pub sha1: String,
    pub metadata: Entry,
}

impl PCloudApi {
    pub async fn get_info_file<I: Into<FileIdentifier>>(
        &self,
        identifier: I,
    ) -> Result<CheckSumFile, Error> {
        let params: FileIdentifier = identifier.into();
        let result: Response<CheckSumFile> =
            self.get_request("checksumfile", &params.to_vec()).await?;
        result.payload()
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::http::PCloudApi;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/checksumfile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
        "result": 0,
        "sha256": "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19",
        "sha1": "5b03ef4fa47ed13f2156ec5395866dadbde4e9dc",
        "metadata": {
                "name": "C61EWBrr2sU16GM4.bin",
                "created": "Sat, 24 Jul 2021 07:38:41 +0000",
                "thumb": false,
                "modified": "Sat, 24 Jul 2021 07:38:41 +0000",
                "isfolder": false,
                "fileid": 5257731387,
                "hash": 9403476549337371523,
                "comments": 0,
                "category": 0,
                "id": "f5257731387",
                "isshared": false,
                "ismine": true,
                "size": 10485760,
                "parentfolderid": 1075398908,
                "contenttype": "application\/octet-stream",
                "icon": "file"
        }
}"#,
            )
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::Test;
        let api = PCloudApi::new(creds, dc);
        let result = api.get_info_file(42).await.unwrap();
        assert_eq!(
            result.sha256,
            "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19"
        );
        m.assert();
    }
}
