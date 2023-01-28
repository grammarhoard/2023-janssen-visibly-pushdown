pub fn identifier_generator(size: usize) -> String {
    let mut s = String::new();
    for i in (1..size+1).rev() {
        s.push_str(&format!("nest{i}:\n  nest{plusone}=nest{plusone} -> nest{plusone}\n", plusone=i - 1));
    }
    s.push_str("nest0:\n  \"a\" -> \"a\"");
    s
}