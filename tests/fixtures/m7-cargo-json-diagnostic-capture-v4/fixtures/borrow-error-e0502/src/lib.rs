pub fn demo() {
    let mut value = 1;
    let shared = &value;
    let mutable = &mut value;
    println!("{} {}", shared, mutable);
}