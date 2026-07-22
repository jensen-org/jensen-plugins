use extism_pdk::*;
use jensen_plugin::{graph, log, ui};
use serde_json::{json, Value};

#[plugin_fn]
pub fn activate() -> FnResult<()> {
    log::info("impact plugin activated");
    Ok(())
}

#[plugin_fn]
pub fn handle_command(input: String) -> FnResult<String> {
    let request: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    let command = request.get("command").and_then(Value::as_str).unwrap_or("");

    let result = match command {
        "impact.check" => {
            let target = request
                .get("args")
                .and_then(|args| args.get("path"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if target.is_empty() {
                json!({ "error": "give a file path to analyze" })
            } else {
                match graph::impact(target) {
                    Ok(impact) => json!({ "target": target, "impact": impact }),
                    Err(error) => json!({ "error": error.to_string() }),
                }
            }
        }
        other => json!({ "error": format!("unknown command '{other}'") }),
    };

    ui::post("impact", result.clone());
    Ok(result.to_string())
}
