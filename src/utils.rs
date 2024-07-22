pub fn contains_noncontiguous(string: &str, pattern: &str) -> bool {
    let mut it = string.split_whitespace();
    for c in pattern.split_whitespace() {
        loop {
            let Some(c2) = it.next() else {
                return false;
            };
            if c2.contains(c) {
                break;
            }
        }
    }
    true
}
