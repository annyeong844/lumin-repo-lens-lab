#[cfg(feature = "bad")]
pub fn gated() {
    let _value: u8 = "feature string";
}

#[cfg(not(feature = "bad"))]
pub fn gated() -> u8 {
    1
}