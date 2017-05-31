extern crate collection_traits;

extern crate persistent_vector;


use collection_traits::*;

use persistent_vector::PersistentVector;


#[test]
fn test() {
    let a = PersistentVector::new();
    let b = a.push(0);
    let c = b.push(1);
    let d = c.push(2);

    assert_eq!(b[0], 0);
    assert_eq!(c.len(), 2);

    assert_eq!(c[0], 0);
    assert_eq!(c[1], 1);
    assert_eq!(c.len(), 2);

    assert_eq!(d[0], 0);
    assert_eq!(d[1], 1);
    assert_eq!(d[2], 2);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_iter() {
    const SIZE: usize = 1024;
    let mut vec = PersistentVector::new();

    for i in 0..SIZE {
        vec = vec.push(i);
    }

    let mut index = 0;
    for value in vec.iter() {
        assert_eq!(value, &index);
        index += 1;
    }
}
