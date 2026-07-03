use verifier::{VerifierResult, verify};

pub const ASSIGN_O: &str = r#"
var x
assume x >= 0
x := x + 1
assert x >= 1
"#;

pub const ASSIGN_1: &str = r#"
var x
assume x >= 0
x := x - 1
assert x >= 1
"#;

pub const ASSIGN_2: &str = r#"
var i, j
assume i > j
i := i + 1
j := j + 1
assert i > j + 1
"#;

pub const ASSIGN_3: &str = r#"
var x
assume (x + 3) * 2 > 6
x := x + 3
x := x * 2
assert x >= 6
"#;

pub const ASSIGN_4: &str = r#"
var a, b, t
t := a
a := b
b := t
assert a == b
"#;

pub const IF_0: &str = r#"
var x, y
if x > 0 then
    y := x
else
    y := 1
end
assert y > 0
"#;

pub const IF_1: &str = r#"
var x, y
assume x >= 0
if x > 0 then
    y := x
else
    y := 0
end
assert y > 0
"#;

pub const IF_2: &str = r#"
var x, y
assume x >= 0
if x > y then
    y := x
end
assert y >= x
"#;

pub const LOOP_TO_0: &str = r#"
var i, n
assume  0 < n
i := n
while 0 < i
invariant i <= n && i >= 0
    i := i - 1
end
assert i == 0
assert i <= n
"#;

pub const LOOP_TO_1: &str = r#"
var i, n
assume 0 < n
i := n
while 0 < i
invariant i <= n && i <= 0
    i := i - 1
end
assert i > 0
"#;

pub const MAX3_0: &str = r#"
var a, b, c, m
if a >= b then
    m := a
else
    m := b
end
if c >= m then
    m := c
end
assert m >= a && m >= b && m >= c
assert m == a || m == b || m == c
"#;

pub const MAX3_1: &str = r#"
var a, b, c, m
if a >= b then
    m := a
else
    m := b
end
if c > m then
    m := c + 1
end
assert m >= a && m >= b && m >= c
assert m == a || m == c || m == c
"#;

pub const MAX3_2: &str = r#"
var a, b, c, m
if a >= b then
    m := a
else
    m := b
end
if c > m then
    m := c + 1
end
assert m >= a && m >= b && m >= c
"#;

pub const MAX3_3: &str = r#"
var a, b, c, m
if a >= b then
    m := a
else
    m := b
end
assert m >= a && m >= b && m >= c
assert m == a || m == b || m == c
"#;

pub const SKIP_0: &str = r#"
var x
assume x >= 0
assert x >= 0
"#;

pub const SKIP_1: &str = r#"
var x
assume x >= 0
assert x == 0
"#;

pub const SLOW_COPY_0: &str = r#"
var x, y, inp, out
assume 0 <= inp
x := inp
y := 0
while 0 < x
invariant x >= 0 && y + x == inp
    x := x - 1
    y := y + 1
end
out := y
assert out == inp
"#;

pub const SLOW_COPY_1: &str = r#"
var x, y, inp, out
assume 0 <= inp
x := inp
y := 0
while 0 < x
invariant TRUE
    x := x - 1
    y := y + 1
end
assert out == inp
"#;

pub const SLOW_COPY_2: &str = r#"
var x, y, inp, out
assume 0 <= inp
x := inp
y := 0
while 0 < x
invariant x >= 0 && y + x == inp
    x := x - 1
    y := y + 2
end
out := y
assert out == inp
"#;

pub const SLOW_SQUARE_0: &str = r#"
var a, i, n
assume 0 <= n
a := 0
i := 0
while i < n
invariant i <= n && a == n * i
    i := i + 1
    a := a + n
end
assert a == n * n
"#;

pub const SLOW_SQUARE_1: &str = r#"
var a, i, n
assume 0 <= n
a := 0
i := 0
while i < n
invariant a == n * i
    i := i + 1
    a := a + n
end
assert a == n * n
"#;

pub const SWAP_0: &str = r#"
var a, b, t
if b < a then
    t := a
    a := b
    b := t
end
assert a <= b
"#;

pub const SWAP_1: &str = r#"
var a, b, t
if b < a then
    t := a
    a := t
end
assert a <= b
"#;

macro_rules! test_example {
    ($name:ident, $example:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(verify($example).result, $expected);
        }
    };
}

test_example!(assign_0, ASSIGN_O, VerifierResult::Correct);
test_example!(assign_1, ASSIGN_1, VerifierResult::Counterexample);
test_example!(assign_2, ASSIGN_2, VerifierResult::Counterexample);
test_example!(assign_3, ASSIGN_3, VerifierResult::Correct);
test_example!(assign_4, ASSIGN_4, VerifierResult::Counterexample);
test_example!(skip_0, SKIP_0, VerifierResult::Correct);
test_example!(skip_1, SKIP_1, VerifierResult::Counterexample);
test_example!(swap_0, SWAP_0, VerifierResult::Correct);
test_example!(swap_1, SWAP_1, VerifierResult::Counterexample);
test_example!(if_0, IF_0, VerifierResult::Correct);
test_example!(if_1, IF_1, VerifierResult::Counterexample);
test_example!(if_2, IF_2, VerifierResult::Correct);
test_example!(slow_copy_0, SLOW_COPY_0, VerifierResult::Correct);
test_example!(slow_copy_1, SLOW_COPY_1, VerifierResult::Counterexample);
test_example!(slow_copy_2, SLOW_COPY_2, VerifierResult::Counterexample);
test_example!(loop_to_0, LOOP_TO_0, VerifierResult::Correct);
test_example!(loop_to_1, LOOP_TO_1, VerifierResult::Counterexample);
test_example!(max3_0, MAX3_0, VerifierResult::Correct);
test_example!(max3_1, MAX3_1, VerifierResult::Counterexample);
test_example!(max3_2, MAX3_2, VerifierResult::Correct);
test_example!(max3_3, MAX3_3, VerifierResult::Counterexample);
test_example!(slow_square_0, SLOW_SQUARE_0, VerifierResult::Correct);
test_example!(slow_square_1, SLOW_SQUARE_1, VerifierResult::Counterexample);
