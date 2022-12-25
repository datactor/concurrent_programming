mod ch01_concurrency_and_parallelism;
mod ch02_basic_programming;
mod ch03_synchronous_processing01;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::sync::mpsc::channel;
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
        assert_eq!(ch02_basic_programming::fun1(), 400);
        ch02_basic_programming::fun3();
        ch02_basic_programming::my_func3();
        ch02_basic_programming::my_func4();
        ch02_basic_programming::my_func8();
        ch02_basic_programming::my_func9();
        ch02_basic_programming::my_func10();
        ch02_basic_programming::my_func11();
        ch02_basic_programming::my_func12();

        ch03_synchronous_processing01::compare_and_swap2();
        ch03_synchronous_processing01::mutex02();

        ch03_synchronous_processing01::some_func4();
        // let mut cnt = AtomicUsize::new(0);
        // cnt = ch03_synchronous_processing01::semaphore_acquire(cnt);
        // ch03_synchronous_processing01::semaphore_release(cnt);
        ch03_synchronous_processing01::some_func6_125p();
        ch03_synchronous_processing01::some_func7_126p();
        ch03_synchronous_processing01::some_func8_127p();
        ch03_synchronous_processing01::some_func9_129p();
    }
}
