// main.rs
pub mod tests;
use tests::IntegrationTest;

fn setup() {
    println!("Setup")
}

fn teardown() {
    println!("Teardown")
}
fn main() {
    setup();

    // Run the tests
    for t in inventory::iter::<IntegrationTest> {
        (t.test_fn)()
    }

    teardown();
}
