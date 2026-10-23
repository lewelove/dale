use serde_json::Value;

#[must_use]
pub fn parse_time(val: Option<&Value>) -> String {
    let now = chrono::Utc::now();
    let dt = match val {
        Some(Value::Number(n)) => n.as_i64().map_or(now, |i| {
            if i > 253_402_300_799 {
                chrono::DateTime::from_timestamp_millis(i).unwrap_or(now)
            } else {
                chrono::DateTime::from_timestamp(i, 0).unwrap_or(now)
            }
        }),
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            trimmed.parse::<i64>().map_or_else(
                |_| chrono::DateTime::parse_from_rfc3339(trimmed).map_or(now, Into::into),
                |i| {
                    if i > 253_402_300_799 {
                        chrono::DateTime::from_timestamp_millis(i).unwrap_or(now)
                    } else {
                        chrono::DateTime::from_timestamp(i, 0).unwrap_or(now)
                    }
                },
            )
        }
        _ => now,
    };
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[must_use]
pub fn toml_to_json(val: toml::Value) -> Value {
    match val {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Number(i.into()),
        toml::Value::Float(f) => {
            serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number)
        }
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, v) in table {
                map.insert(k, toml_to_json(v));
            }
            Value::Object(map)
        }
    }
}

#[must_use]
pub fn json_to_toml(val: Value) -> toml::Value {
    match val {
        Value::Null => toml::Value::String(String::new()),
        Value::Bool(b) => toml::Value::Boolean(b),
        Value::Number(n) => n.as_i64().map_or_else(
            || {
                n.as_f64().map_or_else(
                    || toml::Value::String(n.to_string()),
                    toml::Value::Float,
                )
            },
            toml::Value::Integer,
        ),
        Value::String(s) => toml::Value::String(s),
        Value::Array(arr) => {
            toml::Value::Array(arr.into_iter().map(json_to_toml).collect())
        }
        Value::Object(map) => {
            let mut table = toml::Table::new();
            for (k, v) in map {
                table.insert(k, json_to_toml(v));
            }
            toml::Value::Table(table)
        }
    }
}
