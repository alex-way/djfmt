use criterion::{criterion_group, criterion_main, Criterion};
use djfmt::template_parser;
use std::hint::black_box;

fn benchmark_end_to_end(c: &mut Criterion) {
    c.bench_function("end_to_end", |b| {
        b.iter(|| {
            let mut input = black_box(r#"<html>{{thing}}</html>"#);

            return template_parser::Template::parse(&mut input);
        })
    });
}

criterion_group!(benches, benchmark_end_to_end);
criterion_main!(benches);
