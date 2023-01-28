use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use test_grammar_generator::identifier::identifier_generator;
use vpl_parser_generator::Translator;

pub fn generation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generation speed");
    group.sample_size(10);
    for i in 1usize..=100usize {
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| {
                let input_string = identifier_generator(i * 10);
                Translator::new(black_box(&input_string)).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(bench, generation_speed);
