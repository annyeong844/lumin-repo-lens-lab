use crate::{sha256_bytes, sha256_text};

#[test]
fn sha256_text_uses_lumin_prefixed_hex_shape() {
    assert_eq!(
        sha256_text("lumin"),
        "sha256:9f483025039db444fce444948febe7faae1e2ce3b0001d070b919ccc5d625c00"
    );
}

#[test]
fn sha256_text_matches_byte_hash() {
    assert_eq!(sha256_text("same bytes"), sha256_bytes(b"same bytes"));
}
