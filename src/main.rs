mod threadpool;

fn is_prime(num: usize) -> bool {
    if num < 2 {
        return false;
    }
    if num == 2 {
        return true;
    }
    use integer_sqrt::IntegerSquareRoot;
    for i in 2..(num.integer_sqrt()+1) {
        if num % i == 0 {
            return false;
        }
    }
    return true;
}

fn main() {
    let max_num = 4000000;

    let now = std::time::Instant::now();
    let mut prime_count = 0;
    for i in 1..=max_num {
        // println!("Number {}: prime = {}", i, is_prime(i));
        if is_prime(i) {
            prime_count += 1;
        }
    }
    println!("Number of primes under {}: {}", max_num, prime_count);
    println!("Elapsed time: {} ms", now.elapsed().as_millis());


    use std::sync::{Arc,atomic};

    let now = std::time::Instant::now();
    let mut pool = threadpool::ThreadPool::new(8);
    let prime_count = Arc::new(atomic::AtomicUsize::new(0));
    let mut pool_queue = threadpool::TaskQueue::new();
    for i in 1..=max_num {
        let prime_count_clone = prime_count.clone();
        pool_queue.enqueue(Box::new(move || {
            if is_prime(i) {
                prime_count_clone.fetch_add(1, atomic::Ordering::Relaxed);
            }
        }));
    }
    pool.enqueue_many(pool_queue);
    pool.shutdown_wait();
    println!("Number of primes under {}: {}", max_num, prime_count.load(atomic::Ordering::Relaxed));
    println!("Elapsed time: {} ms", now.elapsed().as_millis());

}
