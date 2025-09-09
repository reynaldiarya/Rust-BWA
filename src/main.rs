fn main() {
    let mut x = 5;
    x = 6;
    println!("The value of x is: {}", x);

    const MAX_POINTS: u32 = 100_000;
    println!("The maximum points are: {}", MAX_POINTS);

    let s1 = String::from("hello");
    let s2 = s1;
    println!("{}, world!", s2);

    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{}', is '{}'.", s1, len);

    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}