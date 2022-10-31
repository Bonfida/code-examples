#[cfg(feature = "benchmarking")]
use bonfida_utils::{bench::BenchRunner, test_name};

#[cfg(feature = "benchmarking")]
pub fn main() {
    let samples = 10;
    let max_schedules = 100;
    let bench_runner = BenchRunner::new(test_name!(), token_vesting::ID);

    let schedules_length = (1..max_schedules)
        .step_by((max_schedules / samples) as usize)
        .collect::<Vec<_>>();

    let mut compute_budget = Vec::with_capacity(99);

    for order_capacity in schedules_length.iter() {
        let res = bench_runner.run(&[order_capacity.to_string()]);
        if res.is_empty() {
            panic!("{}", order_capacity)
        }
        compute_budget.push(res[res.len() - 1]);
    }
    bench_runner.commit(schedules_length, compute_budget);
}

#[cfg(not(feature = "benchmarking"))]
pub fn main() {}
