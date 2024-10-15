use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand::distributions::Alphanumeric;

// Update this line
use vtc::parser::grammar::parse;
use vtc::parser::lexer::tokenize;
use vtc::runtime::Runtime;
use vtc::value::Accessor;

const SMALL_INPUT: &str = r#"
@config:
    $value := 42
    $list := [1, 2, 3, 4, 5]
    $nested := ["a", ["b", ["c", ["d"]]]]
"#;

const LARGE_INPUT: &str = include_str!("../samples/large_config.vtc");

fn generate_random_vtc(size: usize) -> String {
	let mut rng = rand::rngs::StdRng::seed_from_u64(42);
	let mut config = String::new();

	for i in 0..size {
		let namespace = format!("@namespace_{}:\n", i);
		config.push_str(&namespace);

		for _ in 0..rng.gen_range(5..15) {
			let var_name: String = (&mut rng.clone()).sample_iter(&Alphanumeric)
				.take(rng.gen_range(5..15))
				.map(char::from)
				.collect();

			let value = match rng.clone().gen_range(0..5) {
				0 => format!("{}", rng.gen::<i64>()),
				1 => format!("{:.2}", rng.gen::<f64>()),
				2 => format!("\"{}\"", generate_random_string(&mut rng, 20)),
				3 => {
					let list: Vec<i64> = (0..rng.gen_range(5..20)).map(|_| rng.gen()).collect();
					format!("[{}]", list.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(", "))
				}
				_ => {
					if rng.gen_bool(0.5) {
						"True".to_string()
					} else {
						"False".to_string()
					}
				}
			};

			let variable = format!("    ${} := {}\n", var_name, value);
			config.push_str(&variable);
		}
	}

	config
}

fn generate_random_string(rng: &mut rand::rngs::StdRng, len: usize) -> String {
	rng.sample_iter(&Alphanumeric)
		.take(len)
		.map(char::from)
		.collect()
}

fn benchmark_tokenize(c: &mut Criterion) {
	let mut group = c.benchmark_group("Tokenize");

	group.bench_function("Small Input", |b| {
		b.iter(|| tokenize(black_box(SMALL_INPUT)))
	});

	group.bench_function("Large Input", |b| {
		b.iter(|| tokenize(black_box(LARGE_INPUT)))
	});

	for size in [10, 100, 1000].iter() {
		let random_input = generate_random_vtc(*size);
		group.bench_with_input(BenchmarkId::new("Random Input", size), &random_input, |b, i| {
			b.iter(|| tokenize(black_box(i)))
		});
	}

	group.finish();
}

fn benchmark_parse(c: &mut Criterion) {
	let mut group = c.benchmark_group("Parse");

	let (_, small_tokens) = tokenize(SMALL_INPUT).unwrap();
	group.bench_function("Small Input", |b| {
		b.iter(|| parse(black_box(&small_tokens)))
	});

	// let (_, large_tokens) = tokenize(LARGE_INPUT).unwrap();
	// group.bench_function("Large Input", |b| {
	// 	b.iter(|| parse(black_box(&large_tokens)))
	// });

	for size in [10, 100, 1000].iter() {
		let random_input = generate_random_vtc(*size);
		let (_, tokens) = tokenize(&random_input).unwrap();
		group.bench_with_input(BenchmarkId::new("Random Input", size), &tokens, |b, i| {
			b.iter(|| parse(black_box(i)))
		});
	}

	group.finish();
}

fn benchmark_runtime(c: &mut Criterion) {
	let mut group = c.benchmark_group("Runtime");

	group.bench_function("Load Small Input", |b| {
		b.iter(|| {
			let mut runtime = Runtime::new();
			runtime.load_vtc(black_box(SMALL_INPUT)).unwrap();
		})
	});

	group.bench_function("Load Large Input", |b| {
		b.iter(|| {
			let mut runtime = Runtime::new();
			runtime.load_vtc(black_box(LARGE_INPUT)).unwrap();
		})
	});

	for size in [10, 100, 1000].iter() {
		let random_input = generate_random_vtc(*size);
		group.bench_with_input(BenchmarkId::new("Load Random Input", size), &random_input, |b, i| {
			b.iter(|| {
				let mut runtime = Runtime::new();
				runtime.load_vtc(black_box(i)).unwrap();
			})
		});
	}

	let mut runtime = Runtime::new();
	runtime.load_vtc(LARGE_INPUT).unwrap();

	group.bench_function("Get Simple Value", |b| {
		b.iter(|| {
			runtime.get_value(
				black_box("system_config"),
				black_box("max_connections"),
				black_box(&[]),
			).unwrap();
		})
	});

	group.bench_function("Get Nested Value", |b| {
		b.iter(|| {
			runtime.get_value(
				black_box("references"),
				black_box("complex_reference"),
				black_box(&[Accessor::Index(2)]),
			).unwrap();
		})
	});

	group.bench_function("Get Complex Value", |b| {
		b.iter(|| {
			runtime.get_value(
				black_box("calculations"),
				black_box("complex_calculation"),
				black_box(&[]),
			).unwrap();
		})
	});

	group.finish();
}

fn benchmark_intrinsics(c: &mut Criterion) {
	let mut group = c.benchmark_group("Intrinsics");

	let intrinsic_input = r#"
    @math:
        $add := [std_add_int!!, 10, 20]
        $sub := [std_sub_int!!, 50, 30]
        $mul := [std_mul_int!!, 5, 6]
        $div := [std_div_int!!, 100, 4]
        $nested := [std_add_int!!, [std_mul_int!!, 5, 5], [std_sub_int!!, 50, 25]]
    @string:
        $uppercase := [std_to_uppercase!!, "hello world"]
        $concat := [std_concat!!, "Hello, ", "World!"]
    @bitwise:
        $and := [std_bitwise_and!!, 0b1010, 0b1100]
        $not := [std_bitwise_not!!, 0b1010]
    @advanced:
        $base64 := [std_base64_encode!!, "Hello, VTC!"]
        $hash := [std_hash!!, "password123", "sha256"]
    "#;

	let mut runtime = Runtime::new();
	runtime.load_vtc(intrinsic_input).unwrap();

	group.bench_function("Simple Arithmetic Intrinsic", |b| {
		b.iter(|| {
			runtime.get_value(black_box("math"), black_box("add"), black_box(&[])).unwrap();
		})
	});

	group.bench_function("Nested Arithmetic Intrinsic", |b| {
		b.iter(|| {
			runtime.get_value(black_box("math"), black_box("nested"), black_box(&[])).unwrap();
		})
	});

	group.bench_function("String Intrinsic", |b| {
		b.iter(|| {
			runtime.get_value(black_box("string"), black_box("uppercase"), black_box(&[])).unwrap();
		})
	});

	group.bench_function("Bitwise Intrinsic", |b| {
		b.iter(|| {
			runtime.get_value(black_box("bitwise"), black_box("and"), black_box(&[])).unwrap();
		})
	});

	group.bench_function("Advanced Intrinsic", |b| {
		b.iter(|| {
			runtime.get_value(black_box("advanced"), black_box("hash"), black_box(&[])).unwrap();
		})
	});

	group.finish();
}

criterion_group!(
    benches,
    benchmark_tokenize,
    benchmark_parse,
    benchmark_runtime,
    benchmark_intrinsics
);
criterion_main!(benches);