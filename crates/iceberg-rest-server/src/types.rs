//! Helpful types, mostly generated by the axum openapi codegen.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct Prefix(String);

impl Prefix {
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PageToken {
    /// The value is present and not ""
    Present(String),
    /// The value is not present
    NotSpecified,
    /// Specified but empty
    Empty,
}

impl PageToken {
    #[inline]
    #[must_use]
    pub fn new_present(s: String) -> Self {
        if s.is_empty() {
            PageToken::Empty
        } else {
            PageToken::Present(s)
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, PageToken::Empty)
    }

    #[inline]
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, PageToken::NotSpecified)
    }

    #[inline]
    #[must_use]
    pub fn skip_serialize(&self) -> bool {
        matches!(self, PageToken::NotSpecified)
    }
}

impl<'de> Deserialize<'de> for PageToken {
    fn deserialize<D>(deserializer: D) -> Result<PageToken, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) if !s.is_empty() => Ok(PageToken::Present(s)),
            Some(_) => Ok(PageToken::Empty),
            None => Ok(PageToken::NotSpecified),
        }
    }
}

impl Serialize for PageToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PageToken::Present(s) => serializer.serialize_str(s),
            PageToken::NotSpecified => serializer.serialize_none(),
            PageToken::Empty => serializer.serialize_str(""),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum NextPageToken {
    /// The value is present and not "" and not "null"
    NextToken(String),
    /// The value is not present
    Finished,
    /// The server does not support pagination
    /// This omits the `nextPageToken` field in the response
    NotSupported,
}

impl NextPageToken {
    #[inline]
    #[must_use]
    pub fn new_finished() -> Self {
        NextPageToken::Finished
    }

    #[inline]
    #[must_use]
    pub fn new_not_supported() -> Self {
        NextPageToken::NotSupported
    }

    #[inline]
    #[must_use]
    pub fn new_next_token(s: String) -> Self {
        if s.is_empty() {
            NextPageToken::Finished
        } else if s == "null" {
            NextPageToken::NotSupported
        } else {
            NextPageToken::NextToken(s)
        }
    }

    #[inline]
    #[must_use]
    pub fn is_unsupported(&self) -> bool {
        matches!(self, NextPageToken::NotSupported)
    }

    #[inline]
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(self, NextPageToken::Finished)
    }

    #[inline]
    #[must_use]
    pub fn skip_serialize(&self) -> bool {
        matches!(self, NextPageToken::NotSupported)
    }
}

impl Serialize for NextPageToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NextPageToken::NextToken(s) => serializer.serialize_str(s),
            NextPageToken::Finished => serializer.serialize_str("null"),
            NextPageToken::NotSupported => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for NextPageToken {
    fn deserialize<D>(deserializer: D) -> Result<NextPageToken, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) if !s.is_empty() => Ok(NextPageToken::NextToken(s)),
            Some(s) if s == "null" => Ok(NextPageToken::NotSupported),
            Some(_) => Ok(NextPageToken::Finished),
            None => Ok(NextPageToken::NotSupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::Query, http::Request, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_page_token_de() {
        async fn send_request_get_body(query: &str) -> String {
            let body = app()
                .oneshot(
                    Request::builder()
                        .uri(format!("/?{query}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let bytes = body.collect().await.unwrap().to_bytes();
            String::from_utf8(bytes.to_vec()).unwrap()
        }

        fn app() -> Router {
            Router::new().route("/", get(handler))
        }

        async fn handler(Query(params): Query<Params>) -> String {
            format!("{params:?}")
        }

        #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Params {
            page_token: PageToken,
            #[serde(skip_serializing_if = "Option::is_none")]
            parent: Option<String>,
        }

        assert_eq!(
            send_request_get_body("").await,
            r#"Params { page_token: NotSpecified, parent: None }"#
        );

        assert_eq!(
            send_request_get_body("parent=").await,
            r#"Params { page_token: NotSpecified, parent: Some("") }"#
        );

        assert_eq!(
            send_request_get_body("pageToken=123&foo").await,
            r#"Params { page_token: Present("123"), parent: None }"#
        );

        assert_eq!(
            send_request_get_body("pageToken&foo").await,
            r#"Params { page_token: Empty, parent: None }"#
        );

        assert_eq!(
            send_request_get_body("pageToken=&foo").await,
            r#"Params { page_token: Empty, parent: None }"#
        );
    }
}