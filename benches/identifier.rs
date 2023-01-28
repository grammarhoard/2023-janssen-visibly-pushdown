use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use vpa::{nested_id, nested_id_recognizer};

mod vpa {
    use test_grammar_generator::identifier::identifier_generator;
    use vpl_parser_generator::{Translator, Recognizer};

    pub fn nested_id(n: usize) -> (Translator, String) {
        let reg_n = identifier_generator(n);
        let vpa = Translator::new(&reg_n).unwrap();
        let mut test_unit = String::new();
        test_unit.push('a');
        (vpa, test_unit)
    }

    pub fn nested_id_recognizer(n: usize) -> (Recognizer, String) {
        let reg_n = identifier_generator(n);
        let vpa = Recognizer::new(&reg_n).unwrap();
        let mut test_unit = String::new();
        test_unit.push('a');
        (vpa, test_unit)
    }
}

fn bench_nested_id_translator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested identifier/translator");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = nested_id(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.translate(black_box(&test_string)))
        });
    }
    group.finish();
}

fn bench_nested_id_recognizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested identifier/recognizer");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = nested_id_recognizer(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.recognize(black_box(&test_string)))
        });
    }
    group.finish();
}

criterion_group!(
    bench,
    bench_nested_id_recognizer,
    bench_nested_id_translator
);
