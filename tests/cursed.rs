#![feature(asm)]

use reorderable::{OrderToken, reorderable};

fn write_color(color: &str, number: usize) {
    println!("{} {}", color, number);
}

#[test]
fn cursed() {
    let mut red_token = OrderToken::new();
    let mut green_token = OrderToken::new();

    let red_str = "red";
    let green_str = "green";

    let one = 1;
    let two = 2;

    // TODO: figure out a way to make llvm actually reorder this, so that in exchange to being
    // cursed it's also a less big impact on perf? (would guess things to be slower with this than
    // without because extern "C", so…)
    reorderable!(red_token, write_color :: (&str, usize) => (), red_str, one);
    reorderable!(red_token, write_color :: (&str, usize) => (), red_str, two);
    reorderable!(green_token, write_color :: (&str, usize) => (), green_str, one);
    reorderable!(green_token, write_color :: (&str, usize) => (), green_str, two);
}
