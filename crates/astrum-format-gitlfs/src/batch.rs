use astrum_content::{ContentError, ContentRequest, Ctx, View};
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{Map, Value, json};

use crate::{
    JsonBody, asset_path, error, json_response, read_json, read_only, read_only_error,
    require_some, stored, valid_oid,
};

pub const MAX_OBJECTS: usize = 1000;
const BASIC: &str = "basic";
const HASH_ALGORITHM: &str = "sha256";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Download,
    Upload,
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub operation: Operation,
    pub objects: Vec<(Value, i64)>,
}

pub fn parse(body: &Map<String, Value>) -> Result<Request, (StatusCode, String)> {
    let unprocessable = |message: String| (StatusCode::UNPROCESSABLE_ENTITY, message);
    let Some(name) = body.get("operation").and_then(Value::as_str) else {
        return Err(unprocessable("Missing operation".to_owned()));
    };
    let operation = match name.to_ascii_lowercase().as_str() {
        "download" => Operation::Download,
        "upload" => Operation::Upload,
        _ => return Err(unprocessable(format!("Unsupported operation: {name}"))),
    };
    if let Some(algorithm) = body.get("hash_algo").and_then(Value::as_str)
        && algorithm != HASH_ALGORITHM
    {
        return Err((
            StatusCode::CONFLICT,
            format!("Unsupported hash algorithm: {algorithm}"),
        ));
    }
    if let Some(Value::Array(transfers)) = body.get("transfers")
        && !transfers.is_empty()
        && !transfers
            .iter()
            .any(|transfer| transfer.as_str() == Some(BASIC))
    {
        return Err(unprocessable(
            "Only the basic transfer adapter is supported".to_owned(),
        ));
    }
    let Some(Value::Array(objects)) = body.get("objects") else {
        return Err(unprocessable("Missing objects".to_owned()));
    };
    if objects.len() > MAX_OBJECTS {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("A batch request can name at most {MAX_OBJECTS} objects"),
        ));
    }
    Ok(Request {
        operation,
        objects: objects
            .iter()
            .map(|object| {
                let oid = match object.get("oid") {
                    Some(Value::String(oid)) => Value::String(oid.clone()),
                    _ => Value::Null,
                };
                let size = object.get("size").and_then(Value::as_i64).unwrap_or(-1);
                (oid, size)
            })
            .collect(),
    })
}

fn accepted(request: &ContentRequest) -> bool {
    request.header("accept").is_none_or(|accept| {
        accept.contains("git-lfs") || accept.contains("json") || accept.contains("*/*")
    })
}

fn failed(object: &mut Map<String, Value>, code: u16, message: &str) {
    object.insert(
        "error".to_owned(),
        json!({ "code": code, "message": message }),
    );
}

pub async fn batch(
    ctx: &Ctx,
    request: &ContentRequest,
    api: &str,
) -> Result<Response, ContentError> {
    require_some(ctx, View::Read, &request.path)?;
    if !accepted(request) {
        return Ok(error(
            StatusCode::NOT_ACCEPTABLE,
            format!("Responses are sent as {}", crate::MEDIA_TYPE),
        ));
    }
    let body = match read_json(request).await {
        Ok(JsonBody::Object(body)) => body,
        Ok(JsonBody::Missing) => {
            return Ok(error(StatusCode::BAD_REQUEST, "Missing request body"));
        }
        Err(rejected) => return Ok(rejected.response()),
    };
    let parsed = match parse(&body) {
        Ok(parsed) => parsed,
        Err((status, message)) => return Ok(error(status, message)),
    };
    let upload = parsed.operation == Operation::Upload;
    if upload && read_only(ctx) {
        return Ok(read_only_error(ctx));
    }
    let valid: Vec<&str> = parsed
        .objects
        .iter()
        .filter_map(|(oid, _)| oid.as_str())
        .filter(|oid| valid_oid(oid))
        .collect();
    let sizes = stored(ctx, &valid).await?;
    let mut permitted_any = parsed.objects.is_empty();
    let mut objects = Vec::with_capacity(parsed.objects.len());
    for (oid, size) in &parsed.objects {
        let mut object = Map::new();
        object.insert("oid".to_owned(), oid.clone());
        object.insert("size".to_owned(), json!(size));
        let Some(id) = oid.as_str().filter(|oid| valid_oid(oid)) else {
            failed(&mut object, 422, "Invalid object id");
            permitted_any = true;
            objects.push(Value::Object(object));
            continue;
        };
        if *size < 0 {
            failed(&mut object, 422, "Invalid object size");
            permitted_any = true;
            objects.push(Value::Object(object));
            continue;
        }
        let view = if upload { View::Add } else { View::Read };
        let allowed = ctx.allowed(view, &asset_path(id));
        permitted_any |= allowed;
        let href = |path: &str| json!({ "href": format!("{api}{path}") });
        let held = sizes.get(id).copied().filter(|_| allowed);
        match (upload, allowed, held) {
            (false, _, None) => failed(&mut object, 404, "Object does not exist"),
            (false, _, Some(held)) => {
                object.insert("size".to_owned(), json!(held));
                object.insert(
                    "actions".to_owned(),
                    json!({ "download": href(&format!("/objects/{id}")) }),
                );
            }
            (true, false, _) => failed(&mut object, 403, "Not permitted to upload this object"),
            (true, true, Some(held)) => {
                object.insert("size".to_owned(), json!(held));
            }
            (true, true, None) => {
                object.insert(
                    "actions".to_owned(),
                    json!({
                        "upload": href(&format!("/objects/{id}")),
                        "verify": href("/objects/verify"),
                    }),
                );
            }
        }
        objects.push(Value::Object(object));
    }
    if upload && !permitted_any {
        return Ok(error(
            StatusCode::FORBIDDEN,
            "Not permitted to upload to this repository",
        ));
    }
    Ok(json_response(
        StatusCode::OK,
        &json!({ "transfer": BASIC, "objects": objects, "hash_algo": HASH_ALGORITHM }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(value: Value) -> Map<String, Value> {
        match value {
            Value::Object(map) => map,
            _ => unreachable!(),
        }
    }

    #[test]
    fn batch_requests_are_checked_before_their_objects() {
        let ok = parse(&body(json!({
            "operation": "Upload",
            "transfers": ["basic", "tus"],
            "hash_algo": "sha256",
            "objects": [{"oid": "x", "size": 3}, {"size": "big"}]
        })))
        .unwrap();
        assert_eq!(ok.operation, Operation::Upload);
        assert_eq!(ok.objects, vec![(json!("x"), 3), (Value::Null, -1)]);
        let status = |value: Value| parse(&body(value)).unwrap_err().0;
        assert_eq!(
            status(json!({"objects": []})),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status(json!({"operation": "copy", "objects": []})),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status(json!({"operation": "download", "hash_algo": "sha512", "objects": []})),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status(json!({"operation": "download", "transfers": ["tus"], "objects": []})),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status(json!({"operation": "download"})),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        let many: Vec<Value> = (0..=MAX_OBJECTS)
            .map(|_| json!({"oid": "a", "size": 1}))
            .collect();
        assert_eq!(
            status(json!({"operation": "download", "objects": many})),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
}
