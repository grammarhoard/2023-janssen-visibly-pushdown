use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use vpa::{regular_recognizer, regular_translator};

mod vpa {
    use test_grammar_generator::regular::regular_grammar;
    use vpl_parser_generator::{
        Translator, Recognizer,
    };

    pub fn regular_translator(n: usize) -> (Translator, String) {
        let reg_n = regular_grammar(n);
        let vpa = Translator::new(&reg_n).unwrap();
        let mut test_unit = String::new();
        (0..=n)
            .rev()
            .for_each(|i| test_unit.push((('a' as usize) + (i % 26)) as u8 as char));
        test_unit.push('a');
        (vpa, test_unit)
    }

    pub fn regular_recognizer(n: usize) -> (Recognizer, String) {
        let reg_n = regular_grammar(n);
        let vpa = Recognizer::new(&reg_n).unwrap();
        let mut test_unit = String::new();
        (0..=n)
            .rev()
            .for_each(|i| test_unit.push((('a' as usize) + (i % 26)) as u8 as char));
        test_unit.push('a');
        (vpa, test_unit)
    }
}

fn bench_regular_translator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Regular words/translator");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = regular_translator(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.translate(black_box(&test_string)))
        });
    }
    group.finish();
}

fn bench_regular_recognizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Regular words/recognizer");

    for i in 1usize..=100usize {
        let (mut vpa, test_string) = regular_recognizer(i * 100);
        group.bench_function(BenchmarkId::from_parameter(i * 100), |b| {
            b.iter(|| vpa.recognize(black_box(&test_string)))
        });
    }
    group.finish();
}

criterion_group!(bench, bench_regular_recognizer, bench_regular_translator);
