use serde_json::Value;

fn format_toml_value(val: &Value) -> String {
    match val {
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => {
            let formatted: Vec<String> = arr.iter().map(format_toml_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        _ => serde_json::to_string(val).unwrap_or_default(),
    }
}

fn format_toml_value_with_cast(val: &Value, key: &str) -> String {
    if (key == "tracknumber" || key == "discnumber")
        && let Some(s) = val.as_str()
    {
        let clean = s.split('/').next().unwrap_or("").trim();
        if clean.parse::<u32>().is_ok() {
            return clean.to_string();
        }
    }
    format_toml_value(val)
}

pub fn render_toml_block(pool: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut keys: Vec<String> = pool.keys().cloned().collect();
    keys.sort();

    for k in keys {
        if let Some(v) = pool.get(&k) {
            lines.push(format!("{} = {}", k, format_toml_value_with_cast(v, &k)));
        }
    }

    lines
}
