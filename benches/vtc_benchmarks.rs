use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vtc::parser::grammar::parse;
use vtc::parser::lexer::tokenize;
use vtc::runtime::runtime::Runtime;

fn benchmark_tokenize(c: &mut Criterion) {
	let input = r#"
    @test_sample:
        $value_1 := ["hello", "world", "\0"]
        $value_2 := [True, False, %test_sample.value_1->(0..2), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @test_inherit:
        $inherit_1 := [&test_sample.value_1->(1), &test_sample.value_2->(2), "Testing limit in directive test_inherit.", True, False, 1, 2.323, "Hello there!"]
        $value_2 := [True, False, &test_sample.value_1->(0..1), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @memory:
        $ptr_ref := [&test_sample.value_1->[0]]
        $ptr_refs := [ &test_sample.value_1->[0..2] ]
        $nil := Nil
        $single := %nil
    "#;

	c.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(input))));
}

fn benchmark_parse(c: &mut Criterion) {
	let input = r#"
    @test_sample:
        $value_1 := ["hello", "world", "\0"]
        $value_2 := [True, False, %test_sample.value_1->(0..2), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @test_inherit:
        $inherit_1 := [&test_sample.value_1->(1), &test_sample.value_2->(2), "Testing limit in directive test_inherit.", True, False, 1, 2.323, "Hello there!"]
        $value_2 := [True, False, &test_sample.value_1->(0..1), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @memory:
        $ptr_ref := [&test_sample.value_1->[0]]
        $ptr_refs := [ &test_sample.value_1->[0..2] ]
        $nil := Nil
        $single := %nil
    "#;

	let (_, tokens) = tokenize(input).unwrap();
	c.bench_function("parse", |b| b.iter(|| parse(black_box(&tokens))));
}

fn benchmark_runtime(c: &mut Criterion) {
	let input = r#"
    @test_sample:
        $value_1 := ["hello", "world", "\0"]
        $value_2 := [True, False, %test_sample.value_1->(0..2), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @test_inherit:
        $inherit_1 := [&test_sample.value_1->(1), &test_sample.value_2->(2), "Testing limit in directive test_inherit.", True, False, 1, 2.323, "Hello there!"]
        $value_2 := [True, False, &test_sample.value_1->(0..1), False, "Hello", 1, "Testing limit in directive test_inherit." ]

    @memory:
        $ptr_ref := [&test_sample.value_1->[0]]
        $ptr_refs := [ &test_sample.value_1->[0..2] ]
        $nil := Nil
        $single := %nil
    "#;

	c.bench_function("runtime_load", |b| b.iter(|| {
		let mut runtime = Runtime::new();
		runtime.load_vtc(black_box(input)).unwrap();
	}));

	let mut runtime = Runtime::new();
	runtime.load_vtc(input).unwrap();

	c.bench_function("runtime_get_value", |b| b.iter(|| {
		runtime.get_value(
			black_box("test_sample"),
			black_box("value_2"),
			black_box(vtc::parser::ast::ReferenceType::Local),
			black_box(vec![]),
		).unwrap();
	}));
}

criterion_group!(benches, benchmark_tokenize, benchmark_parse, benchmark_runtime);
criterion_main!(benches);