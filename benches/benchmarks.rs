#[macro_use]
extern crate criterion;
extern crate red_mod;

use criterion::Criterion;
use red_mod::rax::*;

fn criterion_benchmark(c: &mut Criterion) {
//    c.bench_function("hash", move |b| {
//        // This will avoid timing the to_vec call.
//        b.iter_with_setup(|| std::collections::HashMap::<u64, &str>::new(), |mut data| hash_insert(data))
//    });
//
//    c.bench_function("btree", move |b| {
//        // This will avoid timing the to_vec call.
//        b.iter_with_setup(|| std::collections::BTreeMap::<u64, &str>::new(), |mut data| btree_insert(data))
//    });
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);