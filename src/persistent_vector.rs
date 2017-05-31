use alloc::arc::Arc;

use core::{mem, ptr, fmt};
use core::ops::Index;

use collection_traits::*;


const SHIFT: usize = 5usize;
const SIZE: usize = 1usize << SHIFT;
const MASK: usize = SIZE - 1usize;


#[derive(Debug)]
enum Node<T> {
    Parent([Option<Arc<Node<T>>>; SIZE]),
    Leaf([Option<Arc<T>>; SIZE]),
}

unsafe fn create_slice<T>() -> [Option<T>; SIZE] {
    let mut slice: [Option<T>; SIZE] = mem::uninitialized();

    for value in &mut slice[..] {
        ptr::write(value, None);
    }

    slice
}

impl<T> Node<T> {

    #[inline(always)]
    fn new_parent() -> Self {
        Node::Parent(unsafe {
            create_slice::<Arc<Node<T>>>()
        })
    }

    #[inline(always)]
    fn new_leaf() -> Self {
        Node::Leaf(unsafe {
            create_slice::<Arc<T>>()
        })
    }

    #[inline]
    fn set_value(&mut self, index: usize, value: Arc<T>) {
        match self {
            &mut Node::Leaf(ref mut slice) => unsafe {
                ptr::write(slice.as_ptr().offset(index as isize) as *mut _, value);
            },
            _ => panic!("trying to set value on Parent Node"),
        }
    }
    #[inline]
    fn set_array(&mut self, index: usize, array: Arc<Node<T>>) {
        match self {
            &mut Node::Parent(ref mut slice) => unsafe {
                ptr::write(slice.as_ptr().offset(index as isize) as *mut _, array);
            },
            _ => panic!("trying to set array on Leaf Node"),
        }
    }

    #[inline]
    fn get_array(&self, index: usize) -> Option<&Arc<Node<T>>> {
        match self {
            &Node::Parent(ref slice) => match slice[index] {
                Some(ref array) => Some(array),
                None => None,
            },
            _ => panic!("trying to get array from Leaf Node"),
        }
    }
    #[inline]
    fn get_value(&self, index: usize) -> Option<&Arc<T>> {
        match self {
            &Node::Leaf(ref slice) => match slice[index] {
                Some(ref value) => Some(value),
                None => None,
            },
            _ => panic!("trying to get value from Parent Node"),
        }
    }

    #[inline]
    fn clone_with_len(&self, len: usize) -> Self {
        match self {
            &Node::Parent(ref slice) => {
                let mut new_slice = unsafe {
                    create_slice::<Arc<Node<T>>>()
                };

                for i in 0..len {
                    match &slice[i] {
                        &Some(ref node) => {
                            new_slice[i] = Some(node.clone());
                        },
                        &None => (),
                    }
                }

                Node::Parent(new_slice)
            },
            &Node::Leaf(ref slice) => {
                let mut new_slice = unsafe {
                    create_slice::<Arc<T>>()
                };

                for i in 0..len {
                    match &slice[i] {
                        &Some(ref node) => {
                            new_slice[i] = Some(node.clone());
                        },
                        &None => (),
                    }
                }

                Node::Leaf(new_slice)
            },
        }
    }
}

impl<T> Clone for Node<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.clone_with_len(SIZE)
    }
}


pub struct PersistentVector<T> {
    root: Option<Arc<Node<T>>>,
    tail: Option<Arc<Node<T>>>,
    len: usize,
    shift: usize,
}

impl<T> PersistentVector<T> {
    #[inline(always)]
    pub fn new() -> Self {
        PersistentVector {
            root: None,
            tail: None,
            len: 0usize,
            shift: SHIFT,
        }
    }

    #[inline]
    fn tail_off(len: usize) -> usize {
        if len < SIZE {
            0
        } else {
            ((len - 1) >> SHIFT) << SHIFT
        }
    }

    #[inline]
    fn find_node(&self, index: usize) -> Option<&Arc<Node<T>>> {
        if index < self.len {
            if index >= Self::tail_off(self.len) {
                match self.tail {
                    Some(ref tail) => Some(&tail),
                    None => None,
                }
            } else {
                let mut node = self.root.as_ref();
                let mut level = self.shift;

                while level > 0usize {
                    let i = (index >> level) & MASK;

                    match node {
                        Some(n) => match &**n {
                            &Node::Parent(ref slice) => {
                                 node = slice[i].as_ref();
                            },
                            _ => return None,
                        },
                        None => return None,
                    }

                    level = level - SHIFT;
                }

                node
            }
        } else {
            None
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        match self.find_node(index) {
            Some(arc) => match &**arc {
                &Node::Leaf(ref s) => s[index & MASK].as_ref().map(|value| &**value),
                _ => None,
            },
            _ => None,
        }
    }

    #[inline]
    fn new_path_set(array: &Arc<Node<T>>, len: usize, index: usize, value: T, level: usize) -> Node<T> {
        let mut new_array = array.clone_with_len(((len - 1) >> level) & MASK);

        if level == 0 {
            new_array.set_value(index & MASK, Arc::new(value));
        } else {
            let sub_index = (index >> level) & MASK;
            let sub_array = array.get_array(sub_index).unwrap();
            new_array.set_array(
                sub_index,
                Arc::new(Self::new_path_set(sub_array, len, index, value, level - SHIFT))
            );
        }

        return new_array;
    }

    #[inline]
    fn new_path(array: Arc<Node<T>>, level: usize) -> Arc<Node<T>> {
        if level == 0 {
            array
        } else {
            let mut new_array = Node::new_parent();
            new_array.set_array(0, Self::new_path(array, level - SHIFT));
            Arc::new(new_array)
        }
    }

    #[inline]
    fn push_tail_empty_root(mut new_array: Node<T>, tail_array: Arc<Node<T>>, len: usize, level: usize) -> Arc<Node<T>> {
        let sub_index = ((len - 1) >> level) & MASK;

        let array_to_insert = if level == SHIFT {
            tail_array
        } else {
            Self::new_path(tail_array, level - SHIFT)
        };

        new_array.set_array(sub_index, array_to_insert);

        Arc::new(new_array)
    }
    #[inline]
    fn push_tail(parent_array: &Arc<Node<T>>, tail_array: Arc<Node<T>>, len: usize, level: usize) -> Arc<Node<T>> {
        let sub_index = ((len - 1) >> level) & MASK;
        let mut new_array = parent_array.clone_with_len(sub_index);

        let array_to_insert = if level == SHIFT {
            tail_array
        } else {
            match parent_array.get_array(sub_index) {
                Some(child) => {
                    Self::push_tail(child, tail_array, len, level - SHIFT)
                },
                None => {
                    Self::new_path(tail_array, level - SHIFT)
                },
            }
        };

        new_array.set_array(sub_index, array_to_insert);

        Arc::new(new_array)
    }

    #[inline]
    unsafe fn push_value_mut(&mut self, value: T) {
        let len = self.len;

        if (len - Self::tail_off(len)) < SIZE {
            let mut tail = &mut *(&**self.tail.as_ref().unwrap() as *const Node<T> as *mut Node<T>);
            tail.set_value(len & MASK, Arc::new(value));
        } else {
            let tail_array = self.tail.as_ref().unwrap().clone();
            let shift = self.shift;
            let mut new_shift = self.shift;

            let new_root = if (len >> SHIFT) > (1 << shift) {
                let mut new_root = Node::new_parent();
                new_root.set_array(0, self.root.as_ref().unwrap().clone());
                new_root.set_array(1, Self::new_path(tail_array, shift));
                new_shift += SHIFT;
                Arc::new(new_root)
            } else {
                match self.root.as_ref() {
                    Some(root) => {
                        Self::push_tail(root, tail_array, len, shift)
                    },
                    None => {
                        Self::push_tail_empty_root(Node::new_parent(), tail_array, len, shift)
                    }
                }
            };

            let mut new_tail = Node::new_leaf();
            new_tail.set_value(0, Arc::new(value));

            self.tail = Some(Arc::new(new_tail));
            self.root = Some(new_root);
            self.shift = new_shift;
        }

        self.len = len + 1;
    }

    #[inline]
    pub fn push(&self, value: T) -> Self {
        let mut vector = self.clone();

        if vector.tail.is_none() {
            vector.tail = Some(Arc::new(Node::new_leaf()));
        }

        unsafe {
            vector.push_value_mut(value);
        }

        vector
    }
}

impl<T> Clone for PersistentVector<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        PersistentVector {
            root: match self.root {
                Some(ref root) => {
                    (&**root).clone();
                    Some(root.clone())
                },
                None => None,
            },
            tail: match self.tail {
                Some(ref tail) => {
                    (&**tail).clone();
                    Some(tail.clone())
                },
                None => None,
            },
            len: self.len,
            shift: self.shift,
        }
    }
}

impl<T> Collection for PersistentVector<T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> Index<usize> for PersistentVector<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).as_ref().unwrap()
    }
}

impl<'a, T: 'a> Iterable<'a, &'a T> for PersistentVector<T> {
    type Iter = Iter<'a, T>;

    #[inline(always)]
    fn iter(&'a self) -> Self::Iter {
        Iter {
            vec: self,
            node: self.find_node(0),
            index: 0,
        }
    }
}


pub struct Iter<'a, T: 'a> {
    vec: &'a PersistentVector<T>,
    node: Option<&'a Arc<Node<T>>>,
    index: usize,
}

unsafe impl<'a, T: 'a + Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for Iter<'a, T> {}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        if (index - PersistentVector::<T>::tail_off(index)) >= SIZE {
            self.node = self.vec.find_node(index);
        }

        match self.node {
            Some(arc) => {
                self.index += 1;

                match &**arc {
                    &Node::Leaf(ref s) => {
                        s[index & MASK].as_ref().map(|value| &**value)
                    },
                    _ => None,
                }
            },
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.vec.len - self.index;
        (size, Some(size))
    }
}
