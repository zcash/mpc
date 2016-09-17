use bn::*;
use crossbeam;

pub fn parallel_all<
    G: Group,
    F: Fn(&[G], &[G]) -> bool + Sync
>
(v1: &[G], v2: &[G], f: F, threads: usize) -> bool
{
    assert_eq!(v1.len(), v2.len());
    let f = &f;

    crossbeam::scope(|scope| {
        let window_size = v1.len() / threads;
        let mut tasks = vec![];
        for i in v1.chunks(window_size).zip(v2.chunks(window_size)) {
            tasks.push(scope.spawn(move || {
                f(i.0, i.1)
            }));
        }

        tasks.into_iter().map(|t| t.join()).all(|r| r)
    })
}

pub fn parallel_two<
    Group1: Group,
    Group2: Group,
    F: Fn(usize, &mut [Group1], &mut [Group2]) + Sync
>
(v1: &mut [Group1], v2: &mut [Group2], f: F, threads: usize)
{
    assert_eq!(v1.len(), v2.len());
    let f = &f;

    crossbeam::scope(|scope| {
        let window_size = v1.len() / threads;
        let mut j = 0;
        for v in v1.chunks_mut(window_size)
                   .zip(v2.chunks_mut(window_size)) 
        {
            scope.spawn(move || {
                f(j, v.0, v.1);
            });

            j += window_size;
        }
    });
}

pub fn parallel<
    G: Group,
    F: Fn(usize, &mut [G]) + Sync
>
(v: &mut [G], f: F, threads: usize)
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

pub fn add_all_to<G: Group>(v: &mut [G], other: &[G]) {
    assert_eq!(v.len(), other.len());

    parallel(v, |mut i, v| {
        for a in v {
            *a = *a + other[i];
            i += 1;
        }
    }, ::THREADS);
}
