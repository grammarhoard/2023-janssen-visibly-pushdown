use criterion::criterion_main;

mod generation;
mod nested;
mod regular;
criterion_main!(regular::bench, nested::bench, generation::bench);
