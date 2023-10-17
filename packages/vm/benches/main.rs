use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{
    mock_backend, mock_env, mock_info, mock_instance_options, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, Checksum, Instance,
    InstanceOptions, Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000; // ~1ms
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000_000; // ~20s, allows many calls on one instance

// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

// Multi-threaded get_instance benchmark
const NO_ITERATIONS: i32 = 10;

static CONTRACT: &[u8] = include_bytes!("../terra19z3qj8lwrhla6x58jt5338e3hktfrn6x63ua4226wk2c7psh62psfghzu7.wasm");

fn bench_instance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instance");

    group.bench_function("Single threaded terra contract", |b| {
        let mut no_executes = 0;
        let iterations = NO_ITERATIONS;

        b.iter(|| {

            while no_executes < iterations {
                // Read the trace and get funny contract ref, for now single contract for test TODO
                let backend = mock_backend(&[]);
                let much_gas: InstanceOptions = InstanceOptions {
                    gas_limit: HIGH_GAS_LIMIT,
                    ..DEFAULT_INSTANCE_OPTIONS
                };


                let mut instance =
                    Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();
                
                /* funny example */
                let info = mock_info("creator", &coins(1000, "earth"));
                let msg = br#"{"owner": "terra17w8udj62rtuuzq2fxl8c8hpg3wdtlcdt7z423d","base_asset": "uusd"}"#;

                let contract_result =
                    call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
                assert!(contract_result.into_result().is_ok());

                let info = mock_info("verifies", &coins(15, "earth"));
                let msg = br#"{
                    "feed_price": {
                      "prices": [
                        [
                          "terra14xsm2wzvu7xaf567r693vgfkhmvfs08l68h4tjj5wjgyn5ky8e2qvzyanh",
                          "0.4852275451743555"
                        ],
                        [
                          "terra1ecgazyd0waaj3g7l9cmy5gulhxkps2gmxu9ghducvuypjq68mq2s5lvsct",
                          "0.48321791205568215"
                        ],
                        [
                          "terra17aj4ty4sz4yhgm08na8drc0v03v2jwr3waxcqrwhajj729zhl7zqnpc0ml",
                          "0.4321740342796268"
                        ],
                        [
                          "terra10aa3zdkrc7jwuf8ekl3zq7e7m42vmzqehcmu74e4egc7xkm5kr2s0muyst",
                          "0.989445"
                        ]
                      ]
                    }
                  }"#;
                let contract_result =
                    call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
                assert!(contract_result.into_result().is_ok());
                
                no_executes += 1;
            }

            black_box(no_executes);
        });
    });

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
        .configure_from_args()
}

criterion_group!(
    name = instance;
    config = make_config();
    targets = bench_instance
);

criterion_main!(instance);
