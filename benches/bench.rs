#![feature(test)]


extern crate test;

extern crate persistent_vector;
extern crate collection_traits;


use test::Bencher;

use collection_traits::*;


const SIZE: usize = 1024;


#[bench]
fn bench_persistent_vector(b: &mut Bencher) {
    use persistent_vector::PersistentVector;

    b.iter(|| {
        let mut a = PersistentVector::new();

        for i in 0..SIZE {
            a = a.push(i);
        }

        a
    });
}
#[bench]
fn bench_std_vec(b: &mut Bencher) {
    b.iter(|| {
        let mut a = Vec::new();

        for i in 0..SIZE {
            a.push_front(i);
        }

        a
    });
}

#[bench]
fn bench_persistent_vector_iter(b: &mut Bencher) {
    use persistent_vector::PersistentVector;

    let mut a = PersistentVector::new();
    let mut index = SIZE;

    for i in 0..SIZE {
        a = a.push(i);
    }

    b.iter(move || {
        index = 0;
        for i in a.iter() {
            assert_eq!(i, &index);
            index += 1;
        }
    });
}
#[bench]
fn bench_std_vec_iter(b: &mut Bencher) {
    let mut a = Vec::new();
    let mut index = SIZE;

    for i in 0..SIZE {
        a.push_back(i);
    }

    b.iter(move || {
        index = 0;
        for i in a.iter() {
            assert_eq!(i, &index);
            index += 1;
        }
    });
}
