// Functions whose names look like effects but whose bodies are pure.
// These exercise the `possible` confidence level only.

pub fn fetch_default_color() -> &'static str {
    // expect: Net=possible
    "blue"
}

pub fn save_to_memory(items: &[&str], item: &str) -> Vec<String> {
    // expect: IO=possible, Mut=possible
    // Returns a NEW Vec; original is not mutated.
    let mut out: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    out.push(item.to_string());
    out
}

pub fn update_label(text: &str) -> String {
    // expect: Mut=possible
    text.to_uppercase()
}

pub fn now_str() -> &'static str {
    // expect: Time=possible
    "now"
}

pub fn download_name(_url: &str) -> &'static str {
    // expect: Net=possible
    "filename.txt"
}
