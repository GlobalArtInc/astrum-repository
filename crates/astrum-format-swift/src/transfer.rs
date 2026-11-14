use astrum_content::Component;
use astrum_content::ContentError;
use astrum_content::Ctx;
use astrum_content::import::{Described, Imported};
use astrum_content::transfer::{Bundle, GenericTransfer};
use axum::http::StatusCode;
use futures_util::future::BoxFuture;

use crate::paths::{Package, Route, route};
use crate::{
    ID, Problem, canonical, import, parse_metadata, refuse_duplicate, refuse_read_only, releases_of,
};

pub(crate) static TRANSFER: GenericTransfer = GenericTransfer {
    plan: import::plan,
    describe: describe_imported,
    after: None,
    admit: Some(admit_bundle),
    replaceable: None,
    carried: &[ID],
};

fn describe_imported(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(import::describe(file))
}

fn admit_bundle<'a>(ctx: &'a Ctx, bundle: &'a Bundle) -> BoxFuture<'a, Result<(), ContentError>> {
    Box::pin(admit(ctx, bundle))
}

fn refused(problem: Problem) -> ContentError {
    match problem.status {
        StatusCode::CONFLICT => ContentError::Conflict(problem.detail),
        _ => ContentError::BadRequest(problem.detail),
    }
}

fn bad(detail: impl Into<String>) -> ContentError {
    ContentError::BadRequest(detail.into())
}

async fn admit(ctx: &Ctx, bundle: &Bundle) -> Result<(), ContentError> {
    refuse_read_only(ctx).map_err(refused)?;
    let package = Package::new(&bundle.namespace, &bundle.name, Some(&bundle.version))
        .ok_or_else(|| bad("the bundle does not name a valid Swift package release"))?;
    let existing: Vec<Component> = releases_of(ctx, &package).await.map_err(refused)?;
    refuse_duplicate(ctx, &existing, &bundle.version).map_err(refused)?;
    let canonical = canonical(&package, &existing);
    let mut archive = None;
    for asset in &bundle.assets {
        let expected = match route(&asset.path) {
            Route::Archive(_) => {
                if archive.replace(asset).is_some() {
                    return Err(bad("the bundle holds more than one source archive"));
                }
                canonical.archive_path()
            }
            Route::Manifest(_, file) => canonical.manifest_path(&file),
            _ => {
                return Err(bad(format!(
                    "{} is neither a source archive nor a manifest",
                    asset.path
                )));
            }
        };
        if asset.path != expected {
            return Err(bad(format!(
                "{} is not where a release of {} at {} is kept",
                asset.path,
                canonical.identifier(),
                bundle.version
            )));
        }
    }
    let Some(archive) = archive else {
        return Err(bad("the bundle holds no source archive"));
    };
    let swift = &bundle.meta[ID];
    if let Some(checksum) = swift.get("checksum").filter(|value| !value.is_null())
        && !checksum
            .as_str()
            .is_some_and(|found| found.eq_ignore_ascii_case(&archive.sha256))
    {
        return Err(bad(
            "the checksum in the bundle meta is not that of the source archive",
        ));
    }
    if let Some(metadata) = swift.get("metadata").filter(|value| !value.is_null()) {
        let bytes = serde_json::to_vec(metadata).map_err(ContentError::internal)?;
        parse_metadata(Some(&bytes)).map_err(refused)?;
    }
    Ok(())
}
