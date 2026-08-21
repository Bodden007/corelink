mod core;
use core::core::CoreLink;
fn main() {
    let core = CoreLink::new();

    println!("{:?}", core.state());
}
