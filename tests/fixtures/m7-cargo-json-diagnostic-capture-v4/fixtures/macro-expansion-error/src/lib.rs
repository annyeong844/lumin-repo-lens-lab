macro_rules! make_bad {
    () => {
        let value: u8 = "macro string";
    };
}

pub fn demo() {
    make_bad!();
}