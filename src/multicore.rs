use bn::*;
use crossbeam;

pub const THREADS: usize = 8;

pub fn mul_all_by<G: Group>(v: &mut [G], c: Fr) {
    crossbeam::scope(|scope| {
    	let window_size = v.len() / THREADS;
        for i in v.chunks_mut(window_size) {
            scope.spawn(move || {
                for i in i {
                    *i = *i * c;
                }
            });
        }
    });
}
