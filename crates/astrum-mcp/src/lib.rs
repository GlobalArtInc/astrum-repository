use async_trait::async_trait;
use serde::Serialize;
use serde_json::{Map, Value, json};

pub const LATEST_VERSION: &str = "2025-06-18";
pub const VERSIONS: [&str; 2] = [LATEST_VERSION, "2025-03-26"];
pub const VERSION_HEADER: &str = "mcp-protocol-version";

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub title: String,
    pub version: String,
    pub instructions: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    pub title: String,
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub title: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Annotations,
}

#[derive(Clone, Debug, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Prompt {
    pub name: String,
    pub title: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug)]
pub enum ToolError {
    Unknown(String),
    InvalidArguments(String),
    Failed(String),
}

impl ToolError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidArguments(message.into())
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed(message.into())
    }
}

#[async_trait]
pub trait Handler: Send + Sync {
    fn tools(&self) -> Vec<Tool>;

    async fn call(&self, name: &str, arguments: Map<String, Value>) -> Result<Value, ToolError>;

    fn prompts(&self) -> Vec<Prompt> {
        Vec::new()
    }

    fn prompt(&self, _name: &str, _arguments: &Map<String, Value>) -> Option<(String, String)> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum Reply {
    Message(Value),
    Accepted,
}

pub fn supported(version: &str) -> bool {
    VERSIONS.contains(&version)
}

pub async fn handle_body(server: &ServerInfo, handler: &dyn Handler, body: &[u8]) -> Reply {
    let message: Value = match serde_json::from_slice(body) {
        Ok(message) => message,
        Err(error) => {
            return Reply::Message(failure(
                Value::Null,
                PARSE_ERROR,
                &format!("the body is not JSON: {error}"),
            ));
        }
    };
    match message {
        Value::Array(batch) => {
            let mut replies = Vec::new();
            for message in batch {
                if let Reply::Message(reply) = handle(server, handler, message).await {
                    replies.push(reply);
                }
            }
            match replies.is_empty() {
                true => Reply::Accepted,
                false => Reply::Message(Value::Array(replies)),
            }
        }
        message => handle(server, handler, message).await,
    }
}

pub async fn handle(server: &ServerInfo, handler: &dyn Handler, message: Value) -> Reply {
    let Value::Object(message) = message else {
        return Reply::Message(failure(
            Value::Null,
            INVALID_REQUEST,
            "a message must be a JSON object",
        ));
    };
    let id = message.get("id").cloned();
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return match id {
            Some(_) if message.contains_key("result") || message.contains_key("error") => {
                Reply::Accepted
            }
            id => Reply::Message(failure(
                id.unwrap_or(Value::Null),
                INVALID_REQUEST,
                "a request needs a method",
            )),
        };
    };
    let params = match message.get("params") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(params)) => params.clone(),
        Some(_) => {
            return Reply::Message(failure(
                id.unwrap_or(Value::Null),
                INVALID_PARAMS,
                "params must be an object",
            ));
        }
    };
    let Some(id) = id else {
        return Reply::Accepted;
    };
    let outcome = match method {
        "initialize" => Ok(initialize(server, &params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": handler.tools() })),
        "tools/call" => call(handler, &params).await,
        "prompts/list" => Ok(json!({ "prompts": handler.prompts() })),
        "prompts/get" => prompt(handler, &params),
        other => Err((
            METHOD_NOT_FOUND,
            format!("the method {other} is not supported"),
        )),
    };
    Reply::Message(match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => failure(id, code, &message),
    })
}

fn initialize(server: &ServerInfo, params: &Map<String, Value>) -> Value {
    let version = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .filter(|version| supported(version))
        .unwrap_or(LATEST_VERSION);
    json!({
        "protocolVersion": version,
        "capabilities": {
            "tools": { "listChanged": false },
            "prompts": { "listChanged": false },
        },
        "serverInfo": {
            "name": server.name,
            "title": server.title,
            "version": server.version,
        },
        "instructions": server.instructions,
    })
}

async fn call(handler: &dyn Handler, params: &Map<String, Value>) -> Result<Value, (i64, String)> {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Err((INVALID_PARAMS, "tools/call needs a tool name".to_owned()));
    };
    let arguments = match params.get("arguments") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(arguments)) => arguments.clone(),
        Some(_) => {
            return Err((INVALID_PARAMS, "arguments must be an object".to_owned()));
        }
    };
    if !handler.tools().iter().any(|tool| tool.name == name) {
        return Err((INVALID_PARAMS, format!("there is no tool named {name}")));
    }
    match handler.call(name, arguments).await {
        Ok(value) => Ok(structured(value)),
        Err(ToolError::Unknown(name)) => {
            Err((INVALID_PARAMS, format!("there is no tool named {name}")))
        }
        Err(ToolError::InvalidArguments(message)) | Err(ToolError::Failed(message)) => Ok(json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true,
        })),
    }
}

fn structured(value: Value) -> Value {
    let value = match value {
        Value::Object(_) => value,
        other => json!({ "result": other }),
    };
    let text = serde_json::to_string_pretty(&value).unwrap_or_default();
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": value,
        "isError": false,
    })
}

fn prompt(handler: &dyn Handler, params: &Map<String, Value>) -> Result<Value, (i64, String)> {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Err((INVALID_PARAMS, "prompts/get needs a prompt name".to_owned()));
    };
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let Some(known) = handler
        .prompts()
        .into_iter()
        .find(|prompt| prompt.name == name)
    else {
        return Err((INVALID_PARAMS, format!("there is no prompt named {name}")));
    };
    if let Some(missing) = known.arguments.iter().find(|argument| {
        argument.required
            && arguments
                .get(&argument.name)
                .and_then(Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
    }) {
        return Err((
            INVALID_PARAMS,
            format!("the prompt {name} needs the argument {}", missing.name),
        ));
    }
    let Some((description, text)) = handler.prompt(name, &arguments) else {
        return Err((INVALID_PARAMS, format!("there is no prompt named {name}")));
    };
    Ok(json!({
        "description": description,
        "messages": [{ "role": "user", "content": { "type": "text", "text": text } }],
    }))
}

fn failure(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

pub fn schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

pub fn text_argument(arguments: &Map<String, Value>, name: &str) -> Option<String> {
    match arguments.get(name) {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

pub fn required_text(arguments: &Map<String, Value>, name: &str) -> Result<String, ToolError> {
    text_argument(arguments, name)
        .ok_or_else(|| ToolError::invalid(format!("the argument {name} is required")))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;

    #[async_trait]
    impl Handler for Echo {
        fn tools(&self) -> Vec<Tool> {
            vec![Tool {
                name: "echo".to_owned(),
                title: "Echo".to_owned(),
                description: "Returns its text".to_owned(),
                input_schema: schema(json!({ "text": { "type": "string" } }), &["text"]),
                annotations: Annotations {
                    title: "Echo".to_owned(),
                    read_only_hint: true,
                    destructive_hint: false,
                    idempotent_hint: true,
                    open_world_hint: false,
                },
            }]
        }

        async fn call(
            &self,
            name: &str,
            arguments: Map<String, Value>,
        ) -> Result<Value, ToolError> {
            match name {
                "echo" => match required_text(&arguments, "text")?.as_str() {
                    "fail" => Err(ToolError::failed("it failed")),
                    text => Ok(json!({ "text": text })),
                },
                other => Err(ToolError::Unknown(other.to_owned())),
            }
        }

        fn prompts(&self) -> Vec<Prompt> {
            vec![Prompt {
                name: "greet".to_owned(),
                title: "Greet".to_owned(),
                description: "Greets someone".to_owned(),
                arguments: vec![PromptArgument {
                    name: "who".to_owned(),
                    description: "Whom to greet".to_owned(),
                    required: true,
                }],
            }]
        }

        fn prompt(&self, name: &str, arguments: &Map<String, Value>) -> Option<(String, String)> {
            (name == "greet").then(|| {
                (
                    "A greeting".to_owned(),
                    format!("Say hello to {}", arguments["who"].as_str().unwrap_or("")),
                )
            })
        }
    }

    fn server() -> ServerInfo {
        ServerInfo {
            name: "astrum".to_owned(),
            title: "Astrum Repository".to_owned(),
            version: "1.0".to_owned(),
            instructions: "Use the tools".to_owned(),
        }
    }

    async fn send(message: Value) -> Value {
        match handle(&server(), &Echo, message).await {
            Reply::Message(reply) => reply,
            Reply::Accepted => panic!("a reply was expected"),
        }
    }

    #[tokio::test]
    async fn initialize_agrees_on_a_version_both_sides_speak() {
        let reply = send(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-03-26", "capabilities": {}}})).await;
        assert_eq!(reply["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(reply["result"]["serverInfo"]["name"], "astrum");
        assert_eq!(reply["result"]["instructions"], "Use the tools");
        assert!(reply["result"]["capabilities"]["tools"].is_object());
        let reply = send(json!({"jsonrpc": "2.0", "id": 2, "method": "initialize", "params": {"protocolVersion": "2099-01-01"}})).await;
        assert_eq!(reply["result"]["protocolVersion"], LATEST_VERSION);
    }

    #[tokio::test]
    async fn notifications_and_replies_get_no_answer() {
        for message in [
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json!({"jsonrpc": "2.0", "id": 4, "result": {}}),
        ] {
            assert_eq!(handle(&server(), &Echo, message).await, Reply::Accepted);
        }
    }

    #[tokio::test]
    async fn tools_are_listed_and_called() {
        let listed = send(json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})).await;
        let tool = &listed["result"]["tools"][0];
        assert_eq!(tool["name"], "echo");
        assert_eq!(tool["inputSchema"]["required"], json!(["text"]));
        assert_eq!(tool["annotations"]["readOnlyHint"], true);

        let called = send(json!({"jsonrpc": "2.0", "id": "a", "method": "tools/call", "params": {"name": "echo", "arguments": {"text": "hi"}}})).await;
        assert_eq!(called["id"], "a");
        assert_eq!(called["result"]["structuredContent"], json!({"text": "hi"}));
        assert_eq!(called["result"]["isError"], false);
        assert!(
            called["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("\"hi\"")
        );

        let failed = send(json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "echo", "arguments": {"text": "fail"}}})).await;
        assert_eq!(failed["result"]["isError"], true);
        assert_eq!(failed["result"]["content"][0]["text"], "it failed");

        let missing = send(json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "echo", "arguments": {}}})).await;
        assert_eq!(missing["result"]["isError"], true);
        assert_eq!(
            missing["result"]["content"][0]["text"],
            "the argument text is required"
        );

        let unknown = send(json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "nothing"}})).await;
        assert_eq!(unknown["error"]["code"], INVALID_PARAMS);
    }

    #[tokio::test]
    async fn prompts_check_their_arguments() {
        let listed = send(json!({"jsonrpc": "2.0", "id": 1, "method": "prompts/list"})).await;
        assert_eq!(listed["result"]["prompts"][0]["name"], "greet");
        let got = send(json!({"jsonrpc": "2.0", "id": 2, "method": "prompts/get", "params": {"name": "greet", "arguments": {"who": "Roman"}}})).await;
        assert_eq!(
            got["result"]["messages"][0]["content"]["text"],
            "Say hello to Roman"
        );
        let missing = send(json!({"jsonrpc": "2.0", "id": 3, "method": "prompts/get", "params": {"name": "greet"}})).await;
        assert_eq!(missing["error"]["code"], INVALID_PARAMS);
    }

    #[tokio::test]
    async fn broken_messages_are_answered_with_errors() {
        let unknown = send(json!({"jsonrpc": "2.0", "id": 1, "method": "resources/list"})).await;
        assert_eq!(unknown["error"]["code"], METHOD_NOT_FOUND);
        let no_method = send(json!({"jsonrpc": "2.0", "id": 2})).await;
        assert_eq!(no_method["error"]["code"], INVALID_REQUEST);
        let bad_params =
            send(json!({"jsonrpc": "2.0", "id": 3, "method": "ping", "params": [1]})).await;
        assert_eq!(bad_params["error"]["code"], INVALID_PARAMS);
        let ping = send(json!({"jsonrpc": "2.0", "id": 4, "method": "ping"})).await;
        assert_eq!(ping["result"], json!({}));
        match handle_body(&server(), &Echo, b"{not json").await {
            Reply::Message(reply) => assert_eq!(reply["error"]["code"], PARSE_ERROR),
            Reply::Accepted => panic!("a parse error needs an answer"),
        }
    }

    #[tokio::test]
    async fn batches_answer_their_requests_only() {
        let body = serde_json::to_vec(&json!([
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 1, "method": "ping"},
        ]))
        .unwrap();
        match handle_body(&server(), &Echo, &body).await {
            Reply::Message(Value::Array(replies)) => {
                assert_eq!(replies.len(), 1);
                assert_eq!(replies[0]["id"], 1);
            }
            other => panic!("one answer was expected, got {other:?}"),
        }
    }
}
