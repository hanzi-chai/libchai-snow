#[test]
fn check_threads() {
    let n = std::thread::available_parallelism().unwrap().get();
    println!("logical cores = {}", n);
    assert!(n > 0);
}
