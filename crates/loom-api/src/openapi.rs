use utoipa::OpenApi;

use crate::error::ErrorBody;
use crate::routes::bundles::{
    BundleDto, CreateBundleRequest, CreateBundleResponse, EditBundleRequest,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::bundles::create,
        crate::routes::bundles::list,
        crate::routes::bundles::get,
        crate::routes::bundles::edit,
        crate::routes::tags::vote,
        crate::routes::tags::aggregate
    ),
    components(schemas(
        BundleDto,
        CreateBundleRequest,
        CreateBundleResponse,
        EditBundleRequest,
        ErrorBody
    )),
    info(title = "Loom API", version = "0.1.0")
)]
pub struct ApiDoc;

pub fn document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
