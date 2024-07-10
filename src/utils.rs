pub fn contains_noncontiguous(string: &String, pattern: &String) -> bool {
  let mut it = string.split_whitespace();
  for c in pattern.split_whitespace() {
      loop {
          let Some(c2) = it.next() else {
              return false;
          };
          if c2.contains(c) {break;}
      }
  }
  true
}