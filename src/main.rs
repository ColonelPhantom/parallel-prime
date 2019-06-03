mod threadpool;

fn is_prime(num: u32) -> bool {
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

    let pool = threadpool::ThreadPool::new(4);
}
