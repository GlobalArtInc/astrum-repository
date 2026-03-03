use astrum_security::{Field, FieldKind, PrivilegeKind};
use axum::Router;
use axum::routing::get;
use serde::Serialize;

use super::UI;
use crate::{ApiError, AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/privileges/types"), get(types))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldXo {
    id: &'static str,
    kind: FieldKind,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<&'static str>>,
    any: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TypeXo {
    id: &'static str,
    fields: Vec<FieldXo>,
}

fn describe(type_id: &'static str, field: &Field) -> FieldXo {
    let values = match field.kind {
        FieldKind::Actions => Some(
            PrivilegeKind::allowed_actions(type_id)
                .iter()
                .map(|action| action.as_str())
                .collect(),
        ),
        FieldKind::Format => Some(
            std::iter::once("*")
                .chain(astrum_repo::FORMATS.iter().map(|format| format.id))
                .collect(),
        ),
        _ => None,
    };
    FieldXo {
        id: field.id,
        kind: field.kind,
        required: field.required,
        values,
        any: matches!(field.kind, FieldKind::Format | FieldKind::Repository),
    }
}

fn common(id: &'static str, required: bool) -> FieldXo {
    FieldXo {
        id,
        kind: FieldKind::Text,
        required,
        values: None,
        any: false,
    }
}

async fn types(caller: Caller) -> Result<axum::Json<Vec<TypeXo>>, ApiError> {
    caller.require("astrum:privileges:read")?;
    Ok(axum::Json(
        PrivilegeKind::TYPES
            .into_iter()
            .map(|type_id| TypeXo {
                id: type_id,
                fields: [common("name", true), common("description", false)]
                    .into_iter()
                    .chain(
                        PrivilegeKind::fields(type_id)
                            .iter()
                            .map(|field| describe(type_id, field)),
                    )
                    .collect(),
            })
            .collect(),
    ))
}
