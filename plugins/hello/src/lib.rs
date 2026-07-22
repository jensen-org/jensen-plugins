use extism_pdk::*;
use jensen_plugin::{log, ui};
use serde_json::{json, Value};

#[plugin_fn]
pub fn activate() -> FnResult<()> {
    log::info("hello plugin activated");
    Ok(())
}

#[plugin_fn]
pub fn handle_command(input: String) -> FnResult<String> {
    let request: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    let command = request.get("command").and_then(Value::as_str).unwrap_or("");

    let result = match command {
        "hello.say" => json!({ "message": "Hello from the sandbox!" }),
        other => json!({ "error": format!("unknown command '{other}'") }),
    };

    ui::post("greeting", result.clone());
    Ok(result.to_string())
}
