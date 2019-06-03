mod threadpool;

fn is_prime(num: usize) -> bool {
    if num < 2 {
        return false;
    }
    if num == 2 {
        return true;
    }
    for i in 2..num {
        if num % i == 0 {
            return false;
        }
    }
    return true;
}

fn main() {
    let mut prime_count = 0;
    let max_num = 100000;
    let now = std::time::Instant::now();
    for i in 1..=max_num {
        // println!("Number {}: prime = {}", i, is_prime(i));
        if is_prime(i) {
            prime_count += 1;
        }
    }
    println!("Number of primes under {}: {}", max_num, prime_count);
    println!("Elapsed time: {} ms", now.elapsed().as_millis());

    let mut pool = threadpool::ThreadPool::new(8);
    let prime_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let max_num = 100000;
    let now = std::time::Instant::now();
    for i in 1..=max_num {
        let prime_count_clone = prime_count.clone();
        pool.enqueue(Box::new(move || {
            if is_prime(i) {
                prime_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }));
    }
    // TODO: wait for the task to finish.
    pool.shutdown_wait();
    println!("Number of primes under {}: {}", max_num, prime_count.load(std::sync::atomic::Ordering::Relaxed));
    println!("Elapsed time: {} ms", now.elapsed().as_millis());

}
