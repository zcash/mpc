use bn::*;
use crossbeam;

pub fn parallel<G: Group, F: Fn(usize, &mut [G]) + Sync>(v: &mut [G], f: F, threads: usize)
{
    let f = &f;

    crossbeam::scope(|scope| {
        let window_size = v.len() / threads;
        let mut j = 0;
        for v in v.chunks_mut(window_size) {
            scope.spawn(move || {
                f(j, v);
            });

            j += window_size;
        }
    });
}

pub fn mul_all_by<G: Group>(v: &mut [G], c: Fr) {
    parallel(v, |_, v| {
        for i in v {
            *i = *i * c;
        }
    }, ::THREADS);
}
