use criterion::{black_box, criterion_group, BenchmarkId, Criterion};

use self::vpa::{nested_recognizer, nested_translator};

mod vpa {
    use test_grammar_generator::nested::nested_grammar;
    use vpl_parser_generator::{
        Translator, Recognizer,
    };

    pub fn nested_translator(n: usize) -> (Translator, String) {
        let nested_n = nested_grammar(n);
        let vpa = Translator::new(&nested_n).unwrap();
        let mut test_unit = String::new();
        (0..n).for_each(|_| test_unit.push('('));
        test_unit.push('a');
        (0..n).for_each(|_| test_unit.push(')'));
        (vpa, test_unit)
    }

    pub fn nested_recognizer(n: usize) -> (Recognizer, String) {
        let nested_n = nested_grammar(n);
        let vpa = Recognizer::new(&nested_n).unwrap();
        let mut test_unit = String::new();
        (0..n).for_each(|_| test_unit.push('('));
        test_unit.push('a');
        (0..n).for_each(|_| test_unit.push(')'));
        (vpa, test_unit)
    }
}

fn bench_nested_translator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested words/translator");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = nested_translator(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.translate(black_box(&test_string)))
        });
    }
    group.finish();
}

fn bench_nested_recognizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested words/recognizer");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = nested_recognizer(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.recognize(black_box(&test_string)))
        });
    }
    group.finish();
}

criterion_group!(bench, bench_nested_recognizer, bench_nested_translator);
