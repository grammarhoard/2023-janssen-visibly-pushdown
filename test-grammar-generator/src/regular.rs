pub fn regular_grammar(size: usize) -> String {
    let mut s = String::new();
    for i in (1..size+1).rev() {
        s.push_str(&format!("reg{i}:\n  \"{cha}\" reg{plusone}=reg{plusone} -> reg{plusone} \"{cha}\"\n", plusone=i - 1, cha=(('a' as u32) + (i % 26) as u32) as u8 as char));
    }
    s.push_str("reg0:\n  \"a\" -> \"a\"");
    s
}