use axum::http::HeaderMap;

pub fn authorized(headers: &HeaderMap, expected: &str) -> bool {
    let Some(value) = headers.get("authorization") else {
        return false;
    };

    let Ok(value) = value.to_str() else {
        return false;
    };

    value == format!("Bearer {expected}")
}

