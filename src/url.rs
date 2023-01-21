use crate::env::{get, API_ORIGIN};

pub fn endpoint(value: &str) -> String {
    return format!("{}/{}", get(API_ORIGIN), value);
}
