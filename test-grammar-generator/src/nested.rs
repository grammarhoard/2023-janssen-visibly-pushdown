pub fn nested_grammar(size: usize) -> String {
    let mut s = String::new();
    for i in (1..size+1).rev() {
        s.push_str(&format!("nested{i}:\n  [\"\\(\" nested{plusone}=nested{plusone} \"\\)\"] -> \"[\" nested{plusone} \"]\"\n", plusone=i - 1));
    }
    s.push_str("nested0:\n  \"a\" -> \"a\"");
    s
}