use criterion::criterion_main;

mod generation;
mod identifier;
mod nested;
mod regular;
criterion_main!(identifier::bench);
