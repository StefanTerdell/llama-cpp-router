use crate::api::result::ApiError;
use reqwest::Method;
use std::fmt::Display;

pub trait ReqwestRequestBuildExt: Sized {
    fn optional_bearer_auth(self, token: Option<impl Display>) -> Self;
}

impl ReqwestRequestBuildExt for reqwest::RequestBuilder {
    fn optional_bearer_auth(self, token: Option<impl Display>) -> Self {
        if let Some(token) = token {
            self.bearer_auth(token)
        } else {
            self
        }
    }
}

pub trait ReqwestResponseExt: Sized {
    fn map_http_error_response(
        self,
        method: Method,
        url: impl Display,
    ) -> impl Future<Output = Result<Self, ApiError>>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn map_http_error_response(
        self,
        method: Method,
        url: impl Display,
    ) -> Result<Self, ApiError> {
        if self.status().is_success() {
            Ok(self)
        } else {
            Err(ApiError::from_http_error_response(method, url, self).await)
        }
    }
}
