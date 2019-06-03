
use instruction::ins;

// need three things:
// generic op definitions
// op definitions with type permutations
// transform rules

ins!(
    fn r#move(src: cxy, dest: xy) {
        src = dest
    },
    fn swap(src: xy, dest: xy) {
        src = dest
    },
    fn raise() {
    },
);

fn main() {
}
