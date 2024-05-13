#![allow(deprecated)]
#![allow(missing_docs)]

use tower::BoxError;

#[deprecated = "use `uhg_custom_appollo_roouter::services::router::Request` instead"]
pub type Request = http::Request<hyper::Body>;
#[deprecated = "use `uhg_custom_appollo_roouter::services::router::Response` instead"]
pub type Response = http::Response<hyper::Body>;
#[deprecated = "use `uhg_custom_appollo_roouter::services::router::BoxService` instead"]
pub type BoxService = tower::util::BoxService<Request, Response, BoxError>;
#[deprecated = "use `uhg_custom_appollo_roouter::services::router::BoxCloneService` instead"]
pub type BoxCloneService = tower::util::BoxCloneService<Request, Response, BoxError>;
#[deprecated = "use `uhg_custom_appollo_roouter::services::router::ServiceResult` instead"]
pub type ServiceResult = Result<Response, BoxError>;
