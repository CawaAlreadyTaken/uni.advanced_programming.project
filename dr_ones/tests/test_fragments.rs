use dr_ones::drone::Dr_One;
use wg_2024::tests;

#[test]
fn fragment_forward() {
    tests::generic_fragment_forward::<Dr_One>();
    println!("fragment_forward");
}

#[test]
fn fragment_drop() {
    tests::generic_fragment_drop::<Dr_One>();
    println!("fragment_drop");
}

#[test]
fn chain_fragment_drop() {
    tests::generic_chain_fragment_drop::<Dr_One>();
}

#[test]
fn chain_fragment_ack() {
    tests::generic_chain_fragment_ack::<Dr_One>();
}
