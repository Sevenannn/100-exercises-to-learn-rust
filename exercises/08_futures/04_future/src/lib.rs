//! TODO: get the code to compile by **re-ordering** the statements
//!  in the `example` function. You're not allowed to change the
//!  `spawner` function nor what each line does in `example`.
//!   You can wrap existing statements in blocks `{}` if needed.
use std::rc::Rc;
use tokio::task::yield_now;

fn spawner() {
    tokio::spawn(example());
}

// Calling drop(non_send) doesn't help
// Because when you create an async block or function,
// Rust captures the entire lifetime of variables used within it, not just the point up to where they are dropped
async fn example() {
    {
        let non_send = Rc::new(1);
        println!("{}", non_send);
    }
    yield_now().await;
}
