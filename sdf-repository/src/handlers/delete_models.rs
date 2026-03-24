use actix_web::{
    HttpResponse, Responder, delete, error::ErrorInternalServerError, http::header::ContentType,
    web,
};
use serde::Deserialize;

use crate::{AppState, error::SdfRepositoryError, models::SdfModelEntry};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteModelQuery {
    lineage: Option<String>,
    min_version: Option<String>,
}

impl DeleteModelQuery {
    fn compare_with_model_entry(
        &self,
        sdf_model_entry: &&SdfModelEntry,
    ) -> actix_web::Result<bool> {
        if sdf_model_entry.lineage != self.lineage {
            return Ok(false);
        }

        if let Some(min_version) = &self.min_version {
            let model_version = semver::Version::parse(&sdf_model_entry.version)
                .map_err(|_| SdfRepositoryError::InternalModelQueryError())?;

            let parsed_min_version = semver::Version::parse(min_version).map_err(|_| {
                SdfRepositoryError::ModelQueryError(format!(
                    "Version {min_version} does not adhere to semantic versioning!"
                ))
            })?;

            return Ok(parsed_min_version <= model_version);
        }

        Ok(true)
    }
}

#[utoipa::path()]
#[delete("/{tail:.*}")]
pub(crate) async fn delete_model_handler(
    data: web::Data<AppState>,
    model_query: web::Query<DeleteModelQuery>,
) -> actix_web::Result<impl Responder> {
    let mut models_entries = data
        .models
        .lock()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

    let (deleted_entries, remaining_entries): (_, Vec<_>) = models_entries
        .iter()
        .cloned()
        .partition(|x| model_query.compare_with_model_entry(&x).unwrap_or(false));

    let response =
        serde_json::to_string(&deleted_entries.iter().map(|x| &x.model).collect::<Vec<_>>())?;

    if deleted_entries.is_empty() {
        return Ok(HttpResponse::NotFound().body("No Model has been deleted."));
    }

    models_entries.clear();

    for remaining_entry in remaining_entries {
        models_entries.push(remaining_entry);
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response))
}
