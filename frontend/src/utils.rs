pub fn truncate(s: &str, max_length: usize) -> String {
    // Returns "string is wa..." from "string is way too long."
    // TODO have to figure out text-ellipsis CSS instead since this
    // doesn't hold up well for screen resizing purposes
    assert!(max_length > 3);
    let mut s = s.to_string();
    if s.len() > max_length {
        s.truncate(max_length - 3);
        s.push_str("...");
    }
    s
}
