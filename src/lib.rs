use std::mem;
use std::cell::{UnsafeCell};
use std::sync::{Arc, atomic::{Ordering, AtomicUsize}};
//use std::io::{self, Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub enum PushAccessError {
    Full,
    BadLen,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PopAccessError {
    Empty,
    BadLen,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PushError {
    Full,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PopError {
    Empty,
}

struct SharedVec<T: Sized> {
    cell: UnsafeCell<Vec<T>>,
}

unsafe impl<T: Sized> Sync for SharedVec<T> {}

impl<T: Sized> SharedVec<T> {
    fn new(data: Vec<T>) -> Self {
        Self { cell: UnsafeCell::new(data) }
    }
    unsafe fn get_ref(&self) -> &Vec<T> {
        self.cell.get().as_ref().unwrap()
    }
    unsafe fn get_mut(&self) -> &mut Vec<T> {
        self.cell.get().as_mut().unwrap()
    }
}

pub struct RingBuffer<T: Sized> {
    data: SharedVec<T>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

pub struct Producer<T> {
    rb: Arc<RingBuffer<T>>,
}

pub struct Consumer<T> {
    rb: Arc<RingBuffer<T>>,
}

impl<T: Sized> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let vec_cap = capacity + 1;
        let mut data = Vec::with_capacity(vec_cap);
        unsafe { data.set_len(vec_cap); }
        Self {
            data: SharedVec::new(data),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn split(self) -> (Producer<T>, Consumer<T>) {
        let arc = Arc::new(self);
        (
            Producer { rb: arc.clone() },
            Consumer { rb: arc },
        )
    }

    pub fn capacity(&self) -> usize {
        unsafe { self.data.get_ref() }.len() - 1
    }
}

impl<T: Sized> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        let data = unsafe { self.data.get_mut() };

        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);
        let len = data.len();
        
        let slices = if head <= tail {
            (head..tail, 0..0)
        } else {
            (head..len, 0..tail)
        };

        let drop = |elem_ref: &mut T| {
            mem::drop(mem::replace(elem_ref, unsafe { mem::uninitialized() }));
        };
        for elem in data[slices.0].iter_mut() {
            drop(elem);
        }
        for elem in data[slices.1].iter_mut() {
            drop(elem);
        }

        unsafe { data.set_len(0); }
    }
}

impl<T: Sized> Producer<T> {
    pub unsafe fn push_access<R, E, F>(&mut self, f: F) -> Result<Result<(usize, R), E>, PushAccessError>
    where R: Sized, E: Sized, F: FnOnce(&mut [T], &mut [T]) -> Result<(usize, R), E> {
        let head = self.rb.head.load(Ordering::SeqCst);
        let tail = self.rb.tail.load(Ordering::SeqCst);
        let len = self.rb.data.get_ref().len();

        let ranges = if tail >= head {
            if head > 0 {
                Ok((tail..len, 0..(head - 1)))
            } else {
                if tail < len - 1 {
                    Ok((tail..(len - 1), 0..0))
                } else {
                    Err(PushAccessError::Full)
                }
            }
        } else {
            if tail < head - 1 {
                Ok((tail..(head - 1), 0..0))
            } else {
                Err(PushAccessError::Full)
            }
        }?;

        let slices = (
            &mut self.rb.data.get_mut()[ranges.0],
            &mut self.rb.data.get_mut()[ranges.1],
        );

        match f(slices.0, slices.1) {
            Ok((n, r)) => {
                if n > slices.0.len() + slices.1.len() {
                    Err(PushAccessError::BadLen)
                } else {
                    let new_tail = (tail + n) % len;
                    self.rb.tail.store(new_tail, Ordering::SeqCst);
                    Ok(Ok((n, r)))
                }
            },
            Err(e) => {
                Ok(Err(e))
            }
        }
    }

    pub fn push(&mut self, elem: T) -> Result<(), (PushError, T)> {
        let mut elem_opt = Some(elem);
        match unsafe { self.push_access(|slice, _| {
            mem::forget(mem::replace(&mut slice[0], elem_opt.take().unwrap()));
            Ok((1, ()))
        }) } {
            Ok(res) => match res {
                Ok((n, ())) => {
                    debug_assert_eq!(n, 1);
                    Ok(())
                },
                Err(()) => unreachable!(),
            },
            Err(e) => match e {
                PushAccessError::Full => Err((PushError::Full, elem_opt.unwrap())),
                PushAccessError::BadLen => unreachable!(),
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.rb.capacity()
    }

    pub fn is_full(&self) -> bool {
        let head = self.rb.head.load(Ordering::SeqCst);
        let tail = self.rb.tail.load(Ordering::SeqCst);
        (tail + 1) % self.capacity() == head
    }
}

impl<T: Sized + Copy> Producer<T> {
    pub fn push_many(&mut self, elems: &[T]) -> Result<usize, PushError> {
        let push_fn = |left: &mut [T], right: &mut [T]| {
            Ok((if elems.len() < left.len() {
                left[0..elems.len()].copy_from_slice(elems);
                elems.len()
            } else {
                left.copy_from_slice(&elems[0..left.len()]);
                if elems.len() < left.len() + right.len() {
                    right[0..(elems.len() - left.len())]
                        .copy_from_slice(&elems[left.len()..elems.len()]);
                    elems.len()
                } else {
                    right.copy_from_slice(&elems[left.len()..elems.len()]);
                    left.len() + right.len()
                }
            }, ()))
        };
        match unsafe { self.push_access(push_fn) } {
            Ok(res) => match res {
                Ok((n, ())) => {
                    Ok(n)
                },
                Err(()) => unreachable!(),
            },
            Err(e) => match e {
                PushAccessError::Full => Err(PushError::Full),
                PushAccessError::BadLen => unreachable!(),
            }
        }
    }
}


impl<T: Sized> Consumer<T> {
    pub unsafe fn pop_access<R, E, F>(&mut self, f: F) -> Result<Result<(usize, R), E>, PopAccessError>
    where R: Sized, E: Sized, F: FnOnce(&mut [T], &mut [T]) -> Result<(usize, R), E> {
        let head = self.rb.head.load(Ordering::SeqCst);
        let tail = self.rb.tail.load(Ordering::SeqCst);
        let len = self.rb.data.get_ref().len();

        let ranges = if head < tail {
            Ok((head..tail, 0..0))
        } else if head > tail {
            Ok((head..len, 0..tail))
        } else {
            Err(PopAccessError::Empty)
        }?;

        let slices = (
            &mut self.rb.data.get_mut()[ranges.0],
            &mut self.rb.data.get_mut()[ranges.1],
        );

        match f(slices.0, slices.1) {
            Ok((n, r)) => {
                if n > slices.0.len() + slices.1.len() {
                    Err(PopAccessError::BadLen)
                } else {
                    let new_head = (head + n) % len;
                    self.rb.head.store(new_head, Ordering::SeqCst);
                    Ok(Ok((n, r)))
                }
            },
            Err(e) => {
                Ok(Err(e))
            }
        }
    }

    pub fn pop(&mut self) -> Result<T, PopError> {
        match unsafe { self.pop_access(|slice, _| {
            let elem = mem::replace(&mut slice[0], mem::uninitialized());
            Ok((1, elem))
        }) } {
            Ok(res) => match res {
                Ok((n, elem)) => {
                    debug_assert_eq!(n, 1);
                    Ok(elem)
                },
                Err(()) => unreachable!(),
            },
            Err(e) => match e {
                PopAccessError::Empty => Err(PopError::Empty),
                PopAccessError::BadLen => unreachable!(),
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.rb.capacity()
    }

    pub fn is_empty(&self) -> bool {
        let head = self.rb.head.load(Ordering::SeqCst);
        let tail = self.rb.tail.load(Ordering::SeqCst);
        head == tail
    }
}

impl<T: Sized + Copy> Consumer<T> {
    pub fn pop_many(&mut self, elems: &mut [T]) -> Result<usize, PopError> {
        let pop_fn = |left: &mut [T], right: &mut [T]| {
            let elems_len = elems.len();
            Ok((if elems_len < left.len() {
                elems.copy_from_slice(&left[0..elems_len]);
                elems_len
            } else {
                elems[0..left.len()].copy_from_slice(left);
                if elems_len < left.len() + right.len() {
                    elems[left.len()..elems_len]
                        .copy_from_slice(&right[0..(elems_len - left.len())]);
                    elems_len
                } else {
                    elems[left.len()..elems_len].copy_from_slice(right);
                    left.len() + right.len()
                }
            }, ()))
        };
        match unsafe { self.pop_access(pop_fn) } {
            Ok(res) => match res {
                Ok((n, ())) => {
                    Ok(n)
                },
                Err(()) => unreachable!(),
            },
            Err(e) => match e {
                PopAccessError::Empty => Err(PopError::Empty),
                PopAccessError::BadLen => unreachable!(),
            }
        }
    }
}


/*
pub trait WriteAccess {
    fn write_access<F>(n: usize, f: F) -> io::Result<usize>
    where F: Fn(&mut [u8]) -> io::Result<usize>;
}

pub trait ReadAccess {
    fn read_access<F>(n: usize, f: F) -> io::Result<usize>
    where F: Fn(&[u8]) -> io::Result<usize>;
}

impl WriteAccess for Producer<u8> {
    fn write_access<F>(n: usize, f: F) -> io::Result<usize>
    where F: Fn(&mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl ReadAccess for Consumer<u8> {
    fn read_access<F>(n: usize, f: F) -> io::Result<usize>
    where F: Fn(&[u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Write for Producer<u8> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for Consumer<u8> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}
*/

#[cfg(test)]
#[macro_use]
extern crate matches;

#[cfg(test)]
mod tests {

    use super::*;

    use std::cell::{Cell};
    use std::thread;

    fn head_tail<T>(rb: &RingBuffer<T>) -> (usize, usize) {
        (rb.head.load(Ordering::SeqCst), rb.tail.load(Ordering::SeqCst))
    }

    #[test]
    fn capacity() {
        let cap = 13;
        let buf = RingBuffer::<i32>::new(cap);
        assert_eq!(buf.capacity(), cap);
    }

    #[test]
    fn split_send() {
        let buf = RingBuffer::<i32>::new(10);
        let (prod, cons) = buf.split();
        
        let pjh = thread::spawn(move || {
            let _ = prod;
        });

        let cjh = thread::spawn(move || {
            let _ = cons;
        });

        pjh.join().unwrap();
        cjh.join().unwrap();
    }

    #[test]
    fn push() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, _) = buf.split();
        

        assert_eq!(head_tail(&prod.rb), (0, 0));

        assert_matches!(prod.push(123), Ok(()));
        assert_eq!(head_tail(&prod.rb), (0, 1));

        assert_matches!(prod.push(234), Ok(()));
        assert_eq!(head_tail(&prod.rb), (0, 2));

        assert_matches!(prod.push(345), Err((PushError::Full, 345)));
        assert_eq!(head_tail(&prod.rb), (0, 2));
    }

    #[test]
    fn pop_empty() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (_, mut cons) = buf.split();


        assert_eq!(head_tail(&cons.rb), (0, 0));

        assert_eq!(cons.pop(), Err(PopError::Empty));
        assert_eq!(head_tail(&cons.rb), (0, 0));
    }

    #[test]
    fn push_pop_one() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();

        let vcap = cap + 1;
        let values = [12, 34, 56, 78, 90];
        assert_eq!(head_tail(&cons.rb), (0, 0));

        for (i, v) in values.iter().enumerate() {
            assert_matches!(prod.push(*v), Ok(()));
            assert_eq!(head_tail(&cons.rb), (i % vcap, (i + 1) % vcap));

            match cons.pop() {
                Ok(w) => assert_eq!(w, *v),
                other => panic!(other),
            }
            assert_eq!(head_tail(&cons.rb), ((i + 1) % vcap, (i + 1) % vcap));

            assert_eq!(cons.pop(), Err(PopError::Empty));
            assert_eq!(head_tail(&cons.rb), ((i + 1) % vcap, (i + 1) % vcap));
        }
    }

    #[test]
    fn push_pop_all() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();

        let vcap = cap + 1;
        let values = [(12, 34, 13), (56, 78, 57), (90, 10, 91)];
        assert_eq!(head_tail(&cons.rb), (0, 0));

        for (i, v) in values.iter().enumerate() {
            assert_matches!(prod.push(v.0), Ok(()));
            assert_eq!(head_tail(&cons.rb), (cap*i % vcap, (cap*i + 1) % vcap));

            assert_matches!(prod.push(v.1), Ok(()));
            assert_eq!(head_tail(&cons.rb), (cap*i % vcap, (cap*i + 2) % vcap));

            match prod.push(v.2) {
                Err((PushError::Full, w)) => assert_eq!(w, v.2),
                other => panic!(other),
            }
            assert_eq!(head_tail(&cons.rb), (cap*i % vcap, (cap*i + 2) % vcap));


            match cons.pop() {
                Ok(w) => assert_eq!(w, v.0),
                other => panic!(other),
            }
            assert_eq!(head_tail(&cons.rb), ((cap*i + 1) % vcap, (cap*i + 2) % vcap));

            match cons.pop() {
                Ok(w) => assert_eq!(w, v.1),
                other => panic!(other),
            }
            assert_eq!(head_tail(&cons.rb), ((cap*i + 2) % vcap, (cap*i + 2) % vcap));

            assert_eq!(cons.pop(), Err(PopError::Empty));
            assert_eq!(head_tail(&cons.rb), ((cap*i + 2) % vcap, (cap*i + 2) % vcap));
        }
    }

    #[derive(Debug)]
    struct Dropper<'a> {
        cnt: &'a Cell<i32>,
    }

    impl<'a> Dropper<'a> {
        fn new(c: &'a Cell<i32>) -> Self {
            Self { cnt: c }
        }
    }

    impl<'a> Drop for Dropper<'a> {
        fn drop(&mut self) {
            self.cnt.set(self.cnt.get() + 1);
        }
    }

    #[test]
    fn drop() {
        let (ca, cb) = (Cell::new(0), Cell::new(0));
        let (da, db) = (Dropper::new(&ca), Dropper::new(&cb));

        let cap = 3;
        let buf = RingBuffer::new(cap);

        {
            let (mut prod, mut cons) = buf.split();

            assert_eq!((ca.get(), cb.get()), (0, 0));

            prod.push(da).unwrap();
            assert_eq!((ca.get(), cb.get()), (0, 0));

            prod.push(db).unwrap();
            assert_eq!((ca.get(), cb.get()), (0, 0));

            cons.pop().unwrap();
            assert_eq!((ca.get(), cb.get()), (1, 0));
        }
        
        assert_eq!((ca.get(), cb.get()), (1, 1));
    }

    #[test]
    fn push_access() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();

        let vs_20 = (123, 456);
        let push_fn_20 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            left[0] = vs_20.0;
            left[1] = vs_20.1;
            Ok((2, ()))
        };

        assert_eq!(unsafe {
            prod.push_access(push_fn_20)
        }.unwrap().unwrap(), (2, ()));

        assert_eq!(cons.pop().unwrap(), vs_20.0);
        assert_eq!(cons.pop().unwrap(), vs_20.1);
        assert_matches!(cons.pop(), Err(PopError::Empty));

        let vs_11 = (123, 456);
        let push_fn_11 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 1);
            assert_eq!(right.len(), 1);
            left[0] = vs_11.0;
            right[0] = vs_11.1;
            Ok((2, ()))
        };

        assert_eq!(unsafe {
            prod.push_access(push_fn_11)
        }.unwrap().unwrap(), (2, ()));

        assert_eq!(cons.pop().unwrap(), vs_11.0);
        assert_eq!(cons.pop().unwrap(), vs_11.1);
        assert_matches!(cons.pop(), Err(PopError::Empty));
    }

    /*
    /// This test doesn't compiles.
    /// And that's good :)
    #[test]
    fn push_access_oref() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, _) = buf.split();

        let mut ovar = 123;
        let mut oref = &mut 123;
        let push_fn_20 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            left[0] = 456;
            oref = &mut left[0];
            Ok((1, ()))
        };

        assert_eq!(unsafe {
            prod.push_access(push_fn_20)
        }.unwrap().unwrap(), (1, ()));

        assert_eq!(*oref, 456);
    }
    */

    #[test]
    fn pop_access_full() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (_, mut cons) = buf.split();

        let dummy_fn = |_l: &mut [i32], _r: &mut [i32]| -> Result<(usize, ()), ()> {
            if true {
                Ok((0, ()))
            } else {
                Err(())
            }
        };
        assert_matches!(unsafe { cons.pop_access(dummy_fn) }, Err(PopAccessError::Empty));
    }

    #[test]
    fn pop_access_empty() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (_, mut cons) = buf.split();

        let dummy_fn = |_l: &mut [i32], _r: &mut [i32]| -> Result<(usize, ()), ()> {
            if true {
                Ok((0, ()))
            } else {
                Err(())
            }
        };
        assert_matches!(unsafe { cons.pop_access(dummy_fn) }, Err(PopAccessError::Empty));
    }

    #[test]
    fn pop_access() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();


        let vs_20 = (123, 456);

        assert_matches!(prod.push(vs_20.0), Ok(()));
        assert_matches!(prod.push(vs_20.1), Ok(()));
        assert_matches!(prod.push(0), Err((PushError::Full, 0)));

        let pop_fn_20 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            assert_eq!(left[0], vs_20.0);
            assert_eq!(left[1], vs_20.1);
            Ok((2, ()))
        };

        assert_eq!(unsafe { cons.pop_access(pop_fn_20) }.unwrap().unwrap(), (2, ()));


        let vs_11 = (123, 456);
        
        assert_matches!(prod.push(vs_11.0), Ok(()));
        assert_matches!(prod.push(vs_11.1), Ok(()));
        assert_matches!(prod.push(0), Err((PushError::Full, 0)));
        
        let pop_fn_11 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 1);
            assert_eq!(right.len(), 1);
            assert_eq!(left[0], vs_11.0);
            assert_eq!(right[0], vs_11.1);
            Ok((2, ()))
        };

        assert_eq!(unsafe { cons.pop_access(pop_fn_11) }.unwrap().unwrap(), (2, ()));

    }

    #[test]
    fn push_access_return() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();

        let push_fn_3 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Ok((3, ()))
        };

        assert_matches!(
            unsafe { prod.push_access(push_fn_3) },
            Err(PushAccessError::BadLen)
        );

        let push_fn_err = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), i32> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Err(123)
        };

        assert_matches!(
            unsafe { prod.push_access(push_fn_err) },
            Ok(Err(123))
        );

        let push_fn_0 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Ok((0, ()))
        };

        assert_matches!(
            unsafe { prod.push_access(push_fn_0) },
            Ok(Ok((0, ())))
        );

        let push_fn_1 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            left[0] = 12;
            Ok((1, ()))
        };

        assert_matches!(
            unsafe { prod.push_access(push_fn_1) },
            Ok(Ok((1, ())))
        );

        let push_fn_2 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 1);
            assert_eq!(right.len(), 0);
            left[0] = 34;
            Ok((1, ()))
        };

        assert_matches!(
            unsafe { prod.push_access(push_fn_2) },
            Ok(Ok((1, ())))
        );

        assert_eq!(cons.pop().unwrap(), 12);
        assert_eq!(cons.pop().unwrap(), 34);
        assert_matches!(cons.pop(), Err(PopError::Empty));
    }

    #[test]
    fn pop_access_return() {
        let cap = 2;
        let buf = RingBuffer::<i32>::new(cap);
        let (mut prod, mut cons) = buf.split();

        assert_matches!(prod.push(12), Ok(()));
        assert_matches!(prod.push(34), Ok(()));
        assert_matches!(prod.push(0), Err((PushError::Full, 0)));

        let pop_fn_3 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Ok((3, ()))
        };

        assert_matches!(
            unsafe { cons.pop_access(pop_fn_3) },
            Err(PopAccessError::BadLen)
        );

        let pop_fn_err = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), i32> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Err(123)
        };

        assert_matches!(
            unsafe { cons.pop_access(pop_fn_err) },
            Ok(Err(123))
        );

        let pop_fn_0 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            Ok((0, ()))
        };

        assert_matches!(
            unsafe { cons.pop_access(pop_fn_0) },
            Ok(Ok((0, ())))
        );

        let pop_fn_1 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 2);
            assert_eq!(right.len(), 0);
            assert_eq!(left[0], 12);
            Ok((1, ()))
        };

        assert_matches!(
            unsafe { cons.pop_access(pop_fn_1) },
            Ok(Ok((1, ())))
        );

        let pop_fn_2 = |left: &mut [i32], right: &mut [i32]| -> Result<(usize, ()), ()> {
            assert_eq!(left.len(), 1);
            assert_eq!(right.len(), 0);
            assert_eq!(left[0], 34);
            Ok((1, ()))
        };

        assert_matches!(
            unsafe { cons.pop_access(pop_fn_2) },
            Ok(Ok((1, ())))
        );
    }
}
