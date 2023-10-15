use set_builder::set;

fn main() {
    set! { x : x <- [1, 2, 3], [4, 5, 6], x != 2 };
}
