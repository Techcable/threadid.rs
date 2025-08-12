pub fn main() {
    let x = threadid::debug::DebugThreadId::current();
    println!("Main thread: {x}")
}
