extern crate criterion;
extern crate reducer;

use criterion::*;
use reducer::*;

#[derive(Debug, Default, Copy, Clone)]
struct Pi(f64, i32);

impl Reducer<i32> for Pi {
    fn reduce(&mut self, n: i32) {
        let Pi(pi, i) = self;

        for _ in 0..n {
            *pi += 4f64 * (-1f64).powi(*i) / (2f64 * f64::from(*i) + 1f64);
            *i += 1;
        }
    }
}

fn reducer(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "pi",
        |b, &i| {
            let mut reducers: (Pi, Pi, Pi, Pi, Pi, Pi, Pi, Pi, Pi, Pi, Pi, Pi) = Default::default();
            b.iter(|| reducers.reduce(i));
        },
        vec![1, 4, 16, 64, 256, 1024],
    );
}

criterion_group!(parallel, reducer);
criterion_main!(parallel);
