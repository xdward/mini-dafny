use verifier::verify;

const SPEC: &str = r#"
var x, y, inp, out
assume 0 <= inp

x := inp
y := 0
while x > 0
invariant x >=0 && y + x == inp
    x := x - 1
    y := y + 1
end
out := y
out := out + 1

assert out == inp
"#;

fn main() {
    println!("{}", verify(SPEC).message);
}
