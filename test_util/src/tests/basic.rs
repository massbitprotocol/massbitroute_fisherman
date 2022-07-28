use super::super::IntegrationTest;

fn basic_test1() {
    println!("Running basic test1");
    assert!(true);
}

fn basic_test2() {
    println!("Running basic test2");
    assert!(true);
}

inventory::submit!(IntegrationTest {
    name: "basic1",
    test_fn: basic_test1
});
inventory::submit!(IntegrationTest {
    name: "basic2",
    test_fn: basic_test2
});
