pub fn truncate(s: &str, max_length: usize) -> String {
    // Returns "string is wa..." from "string is way too long."
    // TODO there should be some CSS like text-ellipses that can accomplish this...
    assert!(max_length > 3);
    let mut s = s.to_string();
    if s.len() > max_length {
        s.truncate(max_length - 3);
        s.push_str("...");
    }
    s
}
