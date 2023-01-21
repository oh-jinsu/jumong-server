use dotenv::dotenv;

pub const CDN_ORIGIN: &str = "CDN_ORIGIN";

pub const API_ORIGIN: &str = "API_ORIGIN";

pub fn init() {
    dotenv().ok();
}

pub fn get(key: &str) -> String {
    return std::env::var(key).expect(key);
}
