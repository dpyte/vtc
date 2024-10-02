use criterion::{black_box, Criterion, criterion_group, criterion_main};

use vtc::parser::ast::Accessor::Range;
use vtc::parser::grammar::parse;
use vtc::parser::lexer::tokenize;
use vtc::runtime::runtime::Runtime;

const INPUT: &str = r#"
@system_config:
    $max_connections := 10000
    $timeout_ms := 30000
    $retry_attempts := 5
    $debug_mode := False
    $allowed_ips := ["192.168.1.1", "10.0.0.1", "172.16.0.1"]
    $config_version := "2.5.1"
    $supported_protocols := ["HTTP", "HTTPS", "FTP", "SFTP", "SSH"]
    $advanced_settings := [
        ["cache_ttl", 3600],
        ["max_payload_size", 5242880],
        ["enable_compression", True],
        ["compression_level", 6]
    ]

@user_data:
    $total_users := 1000000
    $active_users := 750234
    $user_types := ["free", "premium", "enterprise"]
    $user_distribution := [652341, 294652, 53241]
    $average_session_time := 1834.7
    $last_login_timestamps := [1625097600, 1625184000, 1625270400, 1625356800, 1625443200]
    $user_preferences := [
        ["theme", "dark"],
        ["language", "en"],
        ["notifications", True],
        ["auto_save", False]
    ]

@metrics:
    $daily_active_users := [
        523451, 542134, 510234, 536723, 558912, 572134, 543376,
        561234, 589134, 572345, 593412, 602345, 589234, 572134
    ]
    $server_load := [0.65, 0.72, 0.58, 0.81, 0.76, 0.69, 0.73]
    $response_times := [120.5, 118.2, 122.7, 119.8, 121.3, 117.9, 123.1]
    $error_rates := [0.02, 0.015, 0.018, 0.022, 0.019, 0.017, 0.021]
    $performance_matrix := [
        [100, 120, 130, 110, 105],
        [95, 115, 125, 105, 100],
        [105, 125, 135, 115, 110],
        [90, 110, 120, 100, 95]
    ]

@feature_flags:
    $new_ui_enabled := True
    $beta_features := ["quick_search", "voice_commands", "dark_mode"]
    $ab_test_groups := ["control", "variant_a", "variant_b"]
    $rollout_percentage := 0.25
    $max_beta_users := 100000
    $feature_dependencies := [
        ["quick_search", ["indexing_service", "search_algorithm_v2"]],
        ["voice_commands", ["speech_recognition", "nlp_engine"]],
        ["dark_mode", ["theming_engine", "contrast_analyzer"]]
    ]

@localization:
    $supported_languages := ["en", "es", "fr", "de", "ja", "zh", "ru", "ar", "hi", "pt"]
    $default_language := "en"
    $translation_coverage := [1.0, 0.95, 0.92, 0.88, 0.75, 0.70, 0.65, 0.60, 0.55, 0.50]
    $rtl_languages := ["ar", "he", "ur"]
    $language_regions := [
        ["en", ["US", "GB", "AU", "CA"]],
        ["es", ["ES", "MX", "AR", "CO"]],
        ["fr", ["FR", "CA", "BE", "CH"]],
        ["de", ["DE", "AT", "CH"]]
    ]

@performance:
    $cache_size_mb := 5120
    $max_threads := 64
    $connection_pool_size := 1000
    $query_timeout_ms := 5000
    $index_rebuild_interval_hours := 24
    $performance_tuning := [
        ["gc_interval", 3600],
        ["batch_size", 1000],
        ["prefetch_count", 10],
        ["write_buffer_size", 67108864]
    ]

@security:
    $password_min_length := 12
    $password_require_special := True
    $password_require_numbers := True
    $password_require_uppercase := True
    $max_login_attempts := 5
    $lockout_duration_minutes := 30
    $two_factor_auth_enabled := True
    $encryption_settings := [
        ["algorithm", "AES-256-GCM"],
        ["key_size", 256],
        ["iv_size", 12],
        ["tag_size", 16]
    ]

@data_storage:
    $primary_database := "postgres"
    $read_replicas := ["replica1.example.com", "replica2.example.com", "replica3.example.com"]
    $backup_frequency_hours := 6
    $max_backup_age_days := 30
    $blob_storage_provider := "aws_s3"
    $blob_storage_bucket := "example-app-uploads"
    $sharding_strategy := [
        ["user_data", "hash"],
        ["analytics", "range"],
        ["logs", "time-series"]
    ]

@api:
    $rate_limit_per_minute := 1000
    $rate_limit_burst := 50
    $require_api_key := True
    $api_version := "v2"
    $deprecated_endpoints := ["v1/users", "v1/posts", "v1/comments"]
    $cors_allowed_origins := ["https://app.example.com", "https://admin.example.com"]
    $endpoint_timeouts := [
        ["v2/users", 5000],
        ["v2/posts", 3000],
        ["v2/comments", 2000],
        ["v2/search", 10000]
    ]

@monitoring:
    $log_level := "info"
    $error_notification_email := "alerts@example.com"
    $metrics_collection_interval_seconds := 60
    $alert_thresholds := [
        ["cpu_usage", 80],
        ["memory_usage", 90],
        ["disk_usage", 85],
        ["error_rate", 0.05]
    ]

@content:
    $max_upload_size_mb := 100
    $allowed_file_types := ["jpg", "png", "gif", "pdf", "doc", "docx", "xls", "xlsx", "txt"]
    $cdn_url := "https://cdn.example.com"
    $image_resize_dimensions := [[800, 600], [400, 300], [200, 150]]
    $default_compression_quality := 0.8
    $content_delivery_regions := [
        ["na", ["us-east-1", "us-west-2"]],
        ["eu", ["eu-central-1", "eu-west-1"]],
        ["asia", ["ap-southeast-1", "ap-northeast-1"]]
    ]

@search:
    $search_provider := "elasticsearch"
    $elasticsearch_url := "http://search.example.com:9200"
    $index_name := "app_content"
    $search_result_limit := 100
    $minimum_should_match := "75%"
    $boost_fields := ["title^2", "description^1.5", "content^1"]
    $custom_analyzers := [
        ["keyword_analyzer", ["lowercase", "asciifolding"]],
        ["ngram_analyzer", ["lowercase", "ngram"]]
    ]

@machine_learning:
    $model_version := "1.2.3"
    $training_data_path := "/path/to/training/data"
    $feature_importance := [0.35, 0.25, 0.2, 0.15, 0.05]
    $hyperparameters := [
        ["learning_rate", 0.01],
        ["max_depth", 5],
        ["num_leaves", 31],
        ["min_child_samples", 20]
    ]

@references:
    $system_timeout := %system_config.timeout_ms
    $active_user_count := %user_data.active_users
    $current_server_load := %metrics.server_load->(6)
    $password_policy := [
        %security.password_min_length,
        %security.password_require_special,
        %security.password_require_numbers,
        %security.password_require_uppercase
    ]
    $latest_api_version := %api.api_version
    $content_delivery_url := %content.cdn_url
    $ml_model_features := %machine_learning.feature_importance
    $nested_reference := %references.password_policy->(2)
    $complex_reference := [
        %system_config.advanced_settings->(2)->(1),
        %metrics.performance_matrix->(1)->(3),
        %feature_flags.feature_dependencies->(0)->(1)->(0)
    ]

@large_lists:
    $user_ids := [10001, 10002, 10003, 10004, 10005, 10006, 10007, 10008, 10009, 10010]
    $timestamps := [1625097600, 1625184000, 1625270400, 1625356800, 1625443200, 1625529600]
    $random_words := [
        "apple", "banana", "cherry", "date", "elderberry", "fig", "grape",
        "honeydew", "imbe", "jackfruit", "kiwi", "lemon", "mango", "nectarine",
        "orange", "papaya", "quince", "raspberry", "strawberry", "tangerine",
        "ugli", "voavanga", "watermelon", "xigua", "yuzu", "zucchini", "acai",
        "blackberry", "coconut", "dragonfruit", "eggplant", "feijoa", "guava",
        "huckleberry", "ilama", "jambolan", "kumquat", "lime", "mulberry",
        "nance", "olive", "persimmon", "quandong", "rambutan", "soursop",
        "tamarind", "ugni", "vanilla", "wampee", "ximenia", "yam", "ziziphus"
    ]

@nested_data:
    $level1 := [
        "key1",
        ["subkey1", "subvalue1"],
        [
            "subkey2",
            [
                ["subsubkey1", "subsubvalue1"],
                ["subsubkey2", [1, 2, 3, 4, 5]]
            ]
        ],
        [
            "key3",
            [
                ["name", "Item 1"],
                ["value", 100]
            ],
            [
                ["name", "Item 2"],
                ["value", 200]
            ],
            [
                ["name", "Item 3"],
                ["value", 300]
            ]
        ]
    ]
    $deep_nest := [
        "a",
        [
            "b",
            [
                "c",
                [
                    "d",
                    [
                        "e",
                        [
                            "f",
                            [
                                "g",
                                "deep value"
                            ]
                        ]
                    ]
                ]
            ]
        ]
    ]

@complex_types:
    $mixed_list := [1, "two", 3.14, True, [5, 6, 7], ["key", "value"]]
    $nested_list := [
        1,
        [2, [3, [4, [5]]]],
        6,
        [7, 8, [9, 10, [11, 12]]]
    ]
    $function_like := [1, 2, 3, 4, 5]
    $multi_type_calculation := [
        [%system_config.max_connections, %performance.cache_size_mb],
        [%user_data.total_users, %user_data.active_users],
        %metrics.server_load->(2)
    ]
"#;

fn benchmark_tokenize(c: &mut Criterion) {
	c.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(INPUT))));
}

fn benchmark_parse(c: &mut Criterion) {
	let (_, tokens) = tokenize(INPUT).unwrap();
	c.bench_function("parse", |b| b.iter(|| parse(black_box(&tokens))));
}

fn benchmark_runtime(c: &mut Criterion) {
	c.bench_function("runtime_load", |b| b.iter(|| {
		let mut runtime = Runtime::new();
		runtime.load_vtc(black_box(INPUT)).unwrap();
	}));

	let mut runtime = Runtime::new();
	runtime.load_vtc(INPUT).unwrap();

	c.bench_function("runtime_get_value", |b| b.iter(|| {
		runtime.get_value(
			black_box("complex_types"),
			black_box("multi_type_calculation"),
			black_box(&[]),
		).unwrap();
	}));
}

criterion_group!(benches, benchmark_tokenize, benchmark_parse, benchmark_runtime);
criterion_main!(benches);