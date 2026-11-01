use std::collections::HashMap;
use std::rc::Rc;

use raylib::prelude::Color;

use crate::context::Context;
use crate::control::CompiledWord;
use crate::error::ForthError;
use crate::memory::CELL_SIZE;
use crate::stack::Value;

pub type Builtin = fn(&mut Context) -> Result<(), ForthError>;

#[derive(Clone)]
pub enum Word {
    Builtin(Builtin),
    UserDefined(Rc<CompiledWord>),
    Color(Color),
    Address(usize),
    Constant(i64),
    FloatConstant(f64),
    Value(usize),
}

pub struct Dictionary {
    words: HashMap<String, Word>,
}

impl Dictionary {
    pub fn new() -> Self {
        let mut dict = Dictionary {
            words: HashMap::new(),
        };

        // Arithmetic
        dict.add("+", add);
        dict.add("-", subtract);
        dict.add("*", multiply);
        dict.add("/", divide);
        dict.add("mod", modulo);
        dict.add("/mod", slash_mod);
        dict.add("negate", negate);
        dict.add("abs", absolute);
        dict.add("min", minimum);
        dict.add("max", maximum);
        dict.add("1+", one_plus);
        dict.add("1-", one_minus);
        dict.add("2*", two_star);
        dict.add("2/", two_slash);
        dict.add("and", bit_and);
        dict.add("or", bit_or);
        dict.add("xor", bit_xor);
        dict.add("invert", bit_invert);
        dict.add("lshift", left_shift);
        dict.add("rshift", right_shift);
        dict.add("sqrt", square_root);
        dict.add("f-sqrt", fixed_square_root);
        dict.add("sin", fixed_sin);
        dict.add("cos", fixed_cos);
        dict.add("tan", fixed_tan);
        dict.add("f*", fixed_multiply);
        dict.add("f/", fixed_divide);
        dict.add("f+", add);
        dict.add("f-", subtract);
        dict.add("int", fixed_to_integer);
        dict.add("fixed", integer_to_fixed);
        dict.add("random", random);
        dict.add("random-seed", random_seed);

        // Floats and math functions
        dict.add("exp", exponent);
        dict.add("ln", natural_log);
        dict.add("log10", logarithm_10);
        dict.add("log2", logarithm_2);
        dict.add("pow", power);
        dict.add("floor", floor_word);
        dict.add("ceil", ceiling_word);
        dict.add("round", round_word);
        dict.add("trunc", truncate_word);
        dict.add("asin", arc_sine);
        dict.add("acos", arc_cosine);
        dict.add("atan", arc_tangent);
        dict.add("atan2", arc_tangent2);
        dict.add("sinh", hyperbolic_sine);
        dict.add("cosh", hyperbolic_cosine);
        dict.add("tanh", hyperbolic_tangent);
        dict.add("hypot", hypotenuse);
        dict.add("int-of", integer_of);

        // Stack
        dict.add("dup", dup);
        dict.add("drop", drop_forth);
        dict.add("swap", swap_forth);
        dict.add("over", over);
        dict.add("rot", rot);
        dict.add("-rot", minus_rot);
        dict.add("2dup", two_dup);
        dict.add("2drop", two_drop);
        dict.add("nip", nip);
        dict.add("tuck", tuck);
        dict.add("?dup", question_dup);
        dict.add("depth", depth);
        dict.add("pick", pick);
        dict.add(">r", to_return_stack);
        dict.add("r>", from_return_stack);
        dict.add("r@", copy_return_stack);
        dict.add("rdepth", return_depth);

        // Comparison
        dict.add("=", equal_forth);
        dict.add("<>", not_equal_forth);
        dict.add("<", less_than_forth);
        dict.add(">", greater_than_forth);
        dict.add("<=", less_or_equal_forth);
        dict.add(">=", greater_or_equal_forth);
        dict.add("0=", zero_equal_forth);
        dict.add("0<>", zero_not_equal_forth);
        dict.add("0<", zero_less_forth);
        dict.add("0>", zero_greater_forth);
        dict.add("not", zero_equal_forth);

        // Output
        dict.add(".", print_forth);
        dict.add(".s", print_stack);
        dict.add("cr", newline);
        dict.add("space", print_space);
        dict.add("type", print_string);
        dict.add("emit", emit_char);

        // Graphics
        dict.add("draw-rectangle", draw_rectangle);
        dict.add("draw-rectangle-lines", draw_rectangle_lines);
        dict.add("draw-circle", draw_circle);
        dict.add("draw-circle-lines", draw_circle_lines);
        dict.add("draw-line", draw_line);
        dict.add("draw-text", draw_text);
        dict.add("draw-number", draw_number);
        dict.add("text-width", text_width);
        dict.add("string-length", string_length);

        // Input
        dict.add("key-down?", key_down);
        dict.add("key-pressed?", key_pressed);
        dict.add("mouse-x", mouse_x);
        dict.add("mouse-y", mouse_y);
        dict.add("mouse-wheel", mouse_wheel);
        dict.add("mouse-down?", mouse_down);
        dict.add("mouse-pressed?", mouse_pressed);

        // Time
        dict.add("dt", delta_time);
        dict.add("time", elapsed_time);
        dict.add("frame", frame_counter);

        // Memory
        dict.add("@", fetch);
        dict.add("!", store);
        dict.add("+!", add_store);
        dict.add("here", here);
        dict.add("allot", allot);
        dict.add(",", comma);
        dict.add("cells", cells);
        dict.add("cell+", cell_plus);

        // Colors
        dict.add_color("rl-lightgray", Color::LIGHTGRAY);
        dict.add_color("rl-gray", Color::GRAY);
        dict.add_color("rl-darkgray", Color::DARKGRAY);
        dict.add_color("rl-yellow", Color::YELLOW);
        dict.add_color("rl-gold", Color::GOLD);
        dict.add_color("rl-orange", Color::ORANGE);
        dict.add_color("rl-pink", Color::PINK);
        dict.add_color("rl-red", Color::RED);
        dict.add_color("rl-maroon", Color::MAROON);
        dict.add_color("rl-green", Color::GREEN);
        dict.add_color("rl-lime", Color::LIME);
        dict.add_color("rl-darkgreen", Color::DARKGREEN);
        dict.add_color("rl-skyblue", Color::SKYBLUE);
        dict.add_color("rl-blue", Color::BLUE);
        dict.add_color("rl-darkblue", Color::DARKBLUE);
        dict.add_color("rl-purple", Color::PURPLE);
        dict.add_color("rl-violet", Color::VIOLET);
        dict.add_color("rl-darkpurple", Color::DARKPURPLE);
        dict.add_color("rl-beige", Color::BEIGE);
        dict.add_color("rl-brown", Color::BROWN);
        dict.add_color("rl-darkbrown", Color::DARKBROWN);
        dict.add_color("rl-white", Color::WHITE);
        dict.add_color("rl-black", Color::BLACK);
        dict.add_color("rl-magenta", Color::MAGENTA);
        dict.add_color("rl-raywhite", Color::RAYWHITE);

        // Constants
        dict.add_constant("true", -1);
        dict.add_constant("false", 0);
        dict.add_constant("pi", 3142);
        dict.add_float_constant("e", std::f64::consts::E);

        for (index, letter) in ('a'..='z').enumerate() {
            dict.add_constant(&format!("key-{}", letter), 65 + index as i64);
        }

        for digit in 0..=9 {
            dict.add_constant(&format!("key-{}", digit), 48 + digit);
        }

        for (name, code) in SPECIAL_KEYS {
            dict.add_constant(name, *code);
        }

        for (name, button) in MOUSE_BUTTONS {
            dict.add_constant(name, *button);
        }

        dict
    }

    fn add(&mut self, name: &str, function: Builtin) {
        self.words.insert(name.to_string(), Word::Builtin(function));
    }

    fn add_color(&mut self, name: &str, color: Color) {
        self.words.insert(name.to_string(), Word::Color(color));
    }

    pub fn add_constant(&mut self, name: &str, value: i64) {
        self.words.insert(name.to_string(), Word::Constant(value));
    }

    pub fn add_float_constant(&mut self, name: &str, value: f64) {
        self.words
            .insert(name.to_string(), Word::FloatConstant(value));
    }

    pub fn define(&mut self, name: &str, word: Rc<CompiledWord>) {
        self.words.insert(name.to_string(), Word::UserDefined(word));
    }

    pub fn define_address(&mut self, name: &str, address: usize) {
        self.words.insert(name.to_string(), Word::Address(address));
    }

    pub fn define_constant(&mut self, name: &str, value: i64) {
        self.words.insert(name.to_string(), Word::Constant(value));
    }

    pub fn define_value(&mut self, name: &str, address: usize) {
        self.words.insert(name.to_string(), Word::Value(address));
    }

    pub fn define_float_constant(&mut self, name: &str, value: f64) {
        self.words
            .insert(name.to_string(), Word::FloatConstant(value));
    }

    pub fn find(&self, name: &str) -> Option<Word> {
        self.words.get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.words.contains_key(name)
    }

    pub fn is_user_defined(&self, name: &str) -> bool {
        matches!(self.words.get(name), Some(Word::UserDefined(_)))
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

const SPECIAL_KEYS: &[(&str, i64)] = &[
    ("key-space", 32),
    ("key-escape", 256),
    ("key-enter", 257),
    ("key-tab", 258),
    ("key-backspace", 259),
    ("key-insert", 260),
    ("key-delete", 261),
    ("key-right", 262),
    ("key-left", 263),
    ("key-down", 264),
    ("key-up", 265),
    ("key-page-up", 266),
    ("key-page-down", 267),
    ("key-home", 268),
    ("key-end", 269),
    ("key-caps-lock", 280),
    ("key-scroll-lock", 281),
    ("key-num-lock", 282),
    ("key-print-screen", 283),
    ("key-pause", 284),
    ("key-f1", 290),
    ("key-f2", 291),
    ("key-f3", 292),
    ("key-f4", 293),
    ("key-f5", 294),
    ("key-f6", 295),
    ("key-f7", 296),
    ("key-f8", 297),
    ("key-f9", 298),
    ("key-f10", 299),
    ("key-f11", 300),
    ("key-f12", 301),
    ("key-left-shift", 340),
    ("key-left-control", 341),
    ("key-left-alt", 342),
    ("key-right-shift", 344),
    ("key-right-control", 345),
    ("key-right-alt", 346),
    ("key-shift", 340),
    ("key-control", 341),
    ("key-alt", 342),
    ("key-comma", 44),
    ("key-period", 46),
    ("key-minus", 45),
    ("key-equal", 61),
    ("key-slash", 47),
    ("key-semicolon", 59),
    ("key-apostrophe", 39),
    ("key-grave", 96),
    ("key-left-bracket", 91),
    ("key-right-bracket", 93),
    ("key-backslash", 92),
];

const MOUSE_BUTTONS: &[(&str, i64)] = &[
    ("mouse-left", 0),
    ("mouse-right", 1),
    ("mouse-middle", 2),
    ("mouse-side", 3),
    ("mouse-extra", 4),
    ("mouse-forward", 5),
    ("mouse-back", 6),
];

// Arithmetic

/// Promote a stack value to `f64` for a mixed-type operation.
fn as_float(value: Value) -> f64 {
    match value {
        Value::Int(int) => int as f64,
        Value::Float(float) => float,
    }
}

/// Push a truth flag (`-1` or `0`).
fn push_flag(ctx: &mut Context, truth: bool) -> Result<(), ForthError> {
    ctx.push(if truth { -1 } else { 0 });
    Ok(())
}

fn add(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            ctx.push(left.wrapping_add(right));
        }
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left) + as_float(right)));
        }
    }

    Ok(())
}

fn subtract(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            ctx.push(left.wrapping_sub(right));
        }
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left) - as_float(right)));
        }
    }

    Ok(())
}

fn multiply(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            ctx.push(left.wrapping_mul(right));
        }
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left) * as_float(right)));
        }
    }

    Ok(())
}

fn divide(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            if right == 0 {
                return Err(ForthError::new("division by zero"));
            }

            ctx.push(
                left.checked_div(right)
                    .ok_or_else(|| ForthError::new("division overflow"))?,
            );
        }
        // Float division follows IEEE semantics: a zero divisor yields
        // infinity or NaN rather than an error.
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left) / as_float(right)));
        }
    }

    Ok(())
}

fn modulo(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;

    if right == 0 {
        return Err(ForthError::new("division by zero"));
    }

    ctx.push(
        left.checked_rem(right)
            .ok_or_else(|| ForthError::new("division overflow"))?,
    );
    Ok(())
}

fn slash_mod(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;

    if right == 0 {
        return Err(ForthError::new("division by zero"));
    }

    let quotient = left
        .checked_div(right)
        .ok_or_else(|| ForthError::new("division overflow"))?;
    let remainder = left
        .checked_rem(right)
        .ok_or_else(|| ForthError::new("division overflow"))?;

    ctx.push(remainder);
    ctx.push(quotient);
    Ok(())
}

fn negate(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value.wrapping_neg()),
        Value::Float(value) => ctx.push_value(Value::Float(-value)),
    }

    Ok(())
}

fn absolute(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value.wrapping_abs()),
        Value::Float(value) => ctx.push_value(Value::Float(value.abs())),
    }

    Ok(())
}

fn minimum(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            ctx.push(left.min(right));
        }
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left).min(as_float(right))));
        }
    }

    Ok(())
}

fn maximum(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            ctx.push(left.max(right));
        }
        (left, right) => {
            ctx.push_value(Value::Float(as_float(left).max(as_float(right))));
        }
    }

    Ok(())
}

fn one_plus(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value.wrapping_add(1)),
        Value::Float(value) => ctx.push_value(Value::Float(value + 1.0)),
    }

    Ok(())
}

fn one_minus(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value.wrapping_sub(1)),
        Value::Float(value) => ctx.push_value(Value::Float(value - 1.0)),
    }

    Ok(())
}

fn two_star(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.push(value.wrapping_mul(2));
    Ok(())
}

fn two_slash(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.push(value >> 1);
    Ok(())
}

fn bit_and(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;
    ctx.push(left & right);
    Ok(())
}

fn bit_or(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;
    ctx.push(left | right);
    Ok(())
}

fn bit_xor(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;
    ctx.push(left ^ right);
    Ok(())
}

fn bit_invert(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.push(!value);
    Ok(())
}

fn left_shift(ctx: &mut Context) -> Result<(), ForthError> {
    let (value, count) = ctx.pop2()?;

    if !(0..64).contains(&count) {
        return Err(ForthError::new("shift count out of range"));
    }

    ctx.push(value.wrapping_shl(count as u32));
    Ok(())
}

fn right_shift(ctx: &mut Context) -> Result<(), ForthError> {
    let (value, count) = ctx.pop2()?;

    if !(0..64).contains(&count) {
        return Err(ForthError::new("shift count out of range"));
    }

    ctx.push(((value as u64) >> count) as i64);
    Ok(())
}

fn square_root(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => {
            if value < 0 {
                return Err(ForthError::new("sqrt of a negative number"));
            }

            ctx.push((value as f64).sqrt() as i64);
        }
        Value::Float(value) => {
            if value < 0.0 {
                return Err(ForthError::new("sqrt of a negative number"));
            }

            ctx.push_value(Value::Float(value.sqrt()));
        }
    }

    Ok(())
}

fn fixed_square_root(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;

    if value < 0 {
        return Err(ForthError::new("f-sqrt of a negative number"));
    }

    let scaled = value as i128 * 1000;
    ctx.push((scaled as f64).sqrt() as i64);
    Ok(())
}

/// `sin`, `cos`, and `tan` keep their fixed-point behavior for integer
/// arguments (radians in scale-1000) and switch to plain float math when
/// given a float: `1571 sin` and `1.571 sin` mean the same thing.
fn sin_cos_tan(ctx: &mut Context, function: fn(f64) -> f64) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => {
            let radians = value as f64 / 1000.0;
            ctx.push((function(radians) * 1000.0).round() as i64);
        }
        Value::Float(radians) => {
            ctx.push_value(Value::Float(function(radians)));
        }
    }

    Ok(())
}

fn fixed_sin(ctx: &mut Context) -> Result<(), ForthError> {
    sin_cos_tan(ctx, |radians| radians.sin())
}

fn fixed_cos(ctx: &mut Context) -> Result<(), ForthError> {
    sin_cos_tan(ctx, |radians| radians.cos())
}

fn fixed_tan(ctx: &mut Context) -> Result<(), ForthError> {
    sin_cos_tan(ctx, |radians| radians.tan())
}

fn fixed_multiply(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;
    let result = (left as i128 * right as i128) / 1000;
    ctx.push(result as i64);
    Ok(())
}

fn fixed_divide(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2()?;

    if right == 0 {
        return Err(ForthError::new("division by zero"));
    }

    let result = (left as i128 * 1000) / right as i128;
    ctx.push(result as i64);
    Ok(())
}

fn fixed_to_integer(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.push(value / 1000);
    Ok(())
}

fn integer_to_fixed(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.push(value.wrapping_mul(1000));
    Ok(())
}

fn random(ctx: &mut Context) -> Result<(), ForthError> {
    let upper = ctx.pop()?;
    let value = ctx.rng.range(upper);
    ctx.push(value);
    Ok(())
}

fn random_seed(ctx: &mut Context) -> Result<(), ForthError> {
    let seed = ctx.pop()?;
    ctx.rng.seed(seed);
    Ok(())
}

// Stack operations

fn dup(ctx: &mut Context) -> Result<(), ForthError> {
    let value = *ctx
        .stack
        .peek()
        .ok_or_else(|| ForthError::new("stack underflow"))?;
    ctx.push_value(value);
    Ok(())
}

fn drop_forth(ctx: &mut Context) -> Result<(), ForthError> {
    ctx.pop_value()?;
    Ok(())
}

fn swap_forth(ctx: &mut Context) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;
    ctx.push_value(right);
    ctx.push_value(left);
    Ok(())
}

fn over(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx
        .stack
        .second()
        .copied()
        .ok_or_else(|| ForthError::new("stack underflow"))?;
    ctx.push_value(value);
    Ok(())
}

fn rot(ctx: &mut Context) -> Result<(), ForthError> {
    let c = ctx.pop_value()?;
    let b = ctx.pop_value()?;
    let a = ctx.pop_value()?;
    ctx.push_value(b);
    ctx.push_value(c);
    ctx.push_value(a);
    Ok(())
}

fn minus_rot(ctx: &mut Context) -> Result<(), ForthError> {
    let c = ctx.pop_value()?;
    let b = ctx.pop_value()?;
    let a = ctx.pop_value()?;
    ctx.push_value(c);
    ctx.push_value(a);
    ctx.push_value(b);
    Ok(())
}

fn two_dup(ctx: &mut Context) -> Result<(), ForthError> {
    let second = ctx
        .stack
        .second()
        .copied()
        .ok_or_else(|| ForthError::new("stack underflow"))?;
    let top = ctx
        .stack
        .peek()
        .copied()
        .ok_or_else(|| ForthError::new("stack underflow"))?;
    ctx.push_value(second);
    ctx.push_value(top);
    Ok(())
}

fn two_drop(ctx: &mut Context) -> Result<(), ForthError> {
    ctx.pop_value()?;
    ctx.pop_value()?;
    Ok(())
}

fn nip(ctx: &mut Context) -> Result<(), ForthError> {
    let top = ctx.pop_value()?;
    ctx.pop_value()?;
    ctx.push_value(top);
    Ok(())
}

fn tuck(ctx: &mut Context) -> Result<(), ForthError> {
    let top = ctx.pop_value()?;
    let second = ctx.pop_value()?;
    ctx.push_value(top);
    ctx.push_value(second);
    ctx.push_value(top);
    Ok(())
}

fn question_dup(ctx: &mut Context) -> Result<(), ForthError> {
    let value = *ctx
        .stack
        .peek()
        .ok_or_else(|| ForthError::new("stack underflow"))?;

    if value.is_truthy() {
        ctx.push_value(value);
    }

    Ok(())
}

fn depth(ctx: &mut Context) -> Result<(), ForthError> {
    let depth = ctx.stack.len() as i64;
    ctx.push(depth);
    Ok(())
}

fn pick(ctx: &mut Context) -> Result<(), ForthError> {
    let index = ctx.pop()?;

    if index < 0 {
        return Err(ForthError::new("pick index out of range"));
    }

    let depth = ctx.stack.len();

    if index as usize >= depth {
        return Err(ForthError::new("pick index out of range"));
    }

    let value = ctx.stack.as_slice()[depth - 1 - index as usize];
    ctx.push_value(value);
    Ok(())
}

fn to_return_stack(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.return_stack.push(value);
    Ok(())
}

fn from_return_stack(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx
        .return_stack
        .pop()
        .ok_or_else(|| ForthError::new("return stack underflow"))?;
    ctx.push(value);
    Ok(())
}

fn copy_return_stack(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx
        .return_stack
        .last()
        .copied()
        .ok_or_else(|| ForthError::new("return stack underflow"))?;
    ctx.push(value);
    Ok(())
}

fn return_depth(ctx: &mut Context) -> Result<(), ForthError> {
    let depth = ctx.return_stack.len() as i64;
    ctx.push(depth);
    Ok(())
}

// Comparison operations

/// Compare the top two values with a mixed-type promotion: two integers
/// compare exactly; if either operand is a float, both are promoted to
/// `f64` (large integers above 2^53 lose exactness in the mixed case).
fn compare(
    ctx: &mut Context,
    on_ints: fn(i64, i64) -> bool,
    on_floats: fn(f64, f64) -> bool,
) -> Result<(), ForthError> {
    let (left, right) = ctx.pop2_values()?;

    let truth = match (left, right) {
        (Value::Int(left), Value::Int(right)) => on_ints(left, right),
        (left, right) => on_floats(as_float(left), as_float(right)),
    };

    push_flag(ctx, truth)
}

/// Compare one value against zero, same promotion rules as `compare`.
fn compare_zero(
    ctx: &mut Context,
    on_int: fn(i64) -> bool,
    on_float: fn(f64) -> bool,
) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => push_flag(ctx, on_int(value)),
        Value::Float(value) => push_flag(ctx, on_float(value)),
    }
}

fn equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(
        ctx,
        |left, right| left == right,
        |left, right| left == right,
    )
}

fn not_equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(
        ctx,
        |left, right| left != right,
        |left, right| left != right,
    )
}

fn less_than_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(ctx, |left, right| left < right, |left, right| left < right)
}

fn greater_than_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(ctx, |left, right| left > right, |left, right| left > right)
}

fn less_or_equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(
        ctx,
        |left, right| left <= right,
        |left, right| left <= right,
    )
}

fn greater_or_equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare(
        ctx,
        |left, right| left >= right,
        |left, right| left >= right,
    )
}

fn zero_equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare_zero(ctx, |value| value == 0, |value| value == 0.0)
}

fn zero_not_equal_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare_zero(ctx, |value| value != 0, |value| value != 0.0)
}

fn zero_less_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare_zero(ctx, |value| value < 0, |value| value < 0.0)
}

fn zero_greater_forth(ctx: &mut Context) -> Result<(), ForthError> {
    compare_zero(ctx, |value| value > 0, |value| value > 0.0)
}

// Float and math functions

fn exponent(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop_number()?;
    ctx.push_value(Value::Float(value.exp()));
    Ok(())
}

fn natural_log(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop_number()?;
    ctx.push_value(Value::Float(value.ln()));
    Ok(())
}

fn logarithm_10(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop_number()?;
    ctx.push_value(Value::Float(value.log10()));
    Ok(())
}

fn logarithm_2(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop_number()?;
    ctx.push_value(Value::Float(value.log2()));
    Ok(())
}

fn power(ctx: &mut Context) -> Result<(), ForthError> {
    let (base, exponent_value) = ctx.pop2_values()?;

    match (base, exponent_value) {
        (Value::Int(base), Value::Int(exp)) => {
            if exp < 0 {
                return Err(ForthError::new("negative exponent in pow (use floats)"));
            }

            // Matches the language contract: wraps on overflow.
            ctx.push(base.wrapping_pow(exp as u32));
        }
        (base, exp) => {
            ctx.push_value(Value::Float(as_float(base).powf(as_float(exp))));
        }
    }

    Ok(())
}

/// Rounding words keep integers untouched and round floats to another
/// float, so `3.7 round` prints as `4` and stays usable in float math.
fn rounded(ctx: &mut Context, function: fn(f64) -> f64) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value),
        Value::Float(value) => ctx.push_value(Value::Float(function(value))),
    }

    Ok(())
}

fn floor_word(ctx: &mut Context) -> Result<(), ForthError> {
    rounded(ctx, |value| value.floor())
}

fn ceiling_word(ctx: &mut Context) -> Result<(), ForthError> {
    rounded(ctx, |value| value.ceil())
}

fn round_word(ctx: &mut Context) -> Result<(), ForthError> {
    rounded(ctx, |value| value.round())
}

fn truncate_word(ctx: &mut Context) -> Result<(), ForthError> {
    rounded(ctx, |value| value.trunc())
}

fn trig_float(ctx: &mut Context, function: fn(f64) -> f64) -> Result<(), ForthError> {
    let value = ctx.pop_number()?;
    ctx.push_value(Value::Float(function(value)));
    Ok(())
}

fn arc_sine(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.asin())
}

fn arc_cosine(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.acos())
}

fn arc_tangent(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.atan())
}

fn arc_tangent2(ctx: &mut Context) -> Result<(), ForthError> {
    // Stack order is `( x y -- angle )`, so `y` is the top item.
    let (x, y) = ctx.pop2_numbers()?;
    ctx.push_value(Value::Float(y.atan2(x)));
    Ok(())
}

fn hyperbolic_sine(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.sinh())
}

fn hyperbolic_cosine(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.cosh())
}

fn hyperbolic_tangent(ctx: &mut Context) -> Result<(), ForthError> {
    trig_float(ctx, |value| value.tanh())
}

fn hypotenuse(ctx: &mut Context) -> Result<(), ForthError> {
    let (x, y) = ctx.pop2_numbers()?;
    ctx.push_value(Value::Float(x.hypot(y)));
    Ok(())
}

/// `int-of ( value -- integer )`: floats truncate toward zero
/// (`3.7 int-of` gives `3`), integers pass through unchanged.
fn integer_of(ctx: &mut Context) -> Result<(), ForthError> {
    match ctx.pop_value()? {
        Value::Int(value) => ctx.push(value),
        Value::Float(value) => ctx.push(value as i64),
    }

    Ok(())
}

// Output

fn print_forth(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop_value()?;
    println!("{}", value);
    Ok(())
}

fn print_stack(ctx: &mut Context) -> Result<(), ForthError> {
    let depth = ctx.stack.len();
    let values = ctx
        .stack
        .as_slice()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("<{}> {}", depth, values);
    Ok(())
}

fn newline(_ctx: &mut Context) -> Result<(), ForthError> {
    println!();
    Ok(())
}

fn print_space(_ctx: &mut Context) -> Result<(), ForthError> {
    print!(" ");
    Ok(())
}

fn print_string(ctx: &mut Context) -> Result<(), ForthError> {
    let handle = ctx.pop()?;
    let text = ctx.strings.get(handle)?.to_string();
    print!("{}", text);
    Ok(())
}

fn emit_char(ctx: &mut Context) -> Result<(), ForthError> {
    let code = ctx.pop()?;

    if let Some(character) = char::from_u32(code as u32) {
        print!("{}", character);
    }

    Ok(())
}

// Graphics

fn draw_rectangle(ctx: &mut Context) -> Result<(), ForthError> {
    let height = ctx.pop()?;
    let width = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;

    ctx.graphics
        .draw_rectangle(x as i32, y as i32, width as i32, height as i32);
    Ok(())
}

fn draw_rectangle_lines(ctx: &mut Context) -> Result<(), ForthError> {
    let height = ctx.pop()?;
    let width = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;

    ctx.graphics
        .draw_rectangle_lines(x as i32, y as i32, width as i32, height as i32);
    Ok(())
}

fn draw_circle(ctx: &mut Context) -> Result<(), ForthError> {
    let radius = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;

    ctx.graphics.draw_circle(x as i32, y as i32, radius as i32);
    Ok(())
}

fn draw_circle_lines(ctx: &mut Context) -> Result<(), ForthError> {
    let radius = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;

    ctx.graphics
        .draw_circle_lines(x as i32, y as i32, radius as i32);
    Ok(())
}

fn draw_line(ctx: &mut Context) -> Result<(), ForthError> {
    let y2 = ctx.pop()?;
    let x2 = ctx.pop()?;
    let y1 = ctx.pop()?;
    let x1 = ctx.pop()?;

    ctx.graphics
        .draw_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32);
    Ok(())
}

fn draw_text(ctx: &mut Context) -> Result<(), ForthError> {
    let size = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;
    let handle = ctx.pop()?;
    let text = ctx.strings.get(handle)?.to_string();

    ctx.graphics
        .draw_text(text, x as i32, y as i32, size as i32);
    Ok(())
}

fn draw_number(ctx: &mut Context) -> Result<(), ForthError> {
    let size = ctx.pop()?;
    let y = ctx.pop()?;
    let x = ctx.pop()?;
    let value = ctx.pop_value()?;

    ctx.graphics
        .draw_text(value.to_string(), x as i32, y as i32, size as i32);
    Ok(())
}

fn text_width(ctx: &mut Context) -> Result<(), ForthError> {
    let size = ctx.pop()?;
    let handle = ctx.pop()?;
    let text = ctx.strings.get(handle)?;
    let width = text.chars().count() as i64 * size / 2;
    ctx.push(width);
    Ok(())
}

fn string_length(ctx: &mut Context) -> Result<(), ForthError> {
    let handle = ctx.pop()?;
    let length = ctx.strings.get(handle)?.chars().count() as i64;
    ctx.push(length);
    Ok(())
}

// Input

fn key_down(ctx: &mut Context) -> Result<(), ForthError> {
    let key = ctx.pop()? as i32;
    let down = ctx.input.key_down(key);
    ctx.push(if down { -1 } else { 0 });
    Ok(())
}

fn key_pressed(ctx: &mut Context) -> Result<(), ForthError> {
    let key = ctx.pop()? as i32;
    let pressed = ctx.input.key_pressed(key);
    ctx.push(if pressed { -1 } else { 0 });
    Ok(())
}

fn mouse_x(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.input.mouse_x as i64;
    ctx.push(value);
    Ok(())
}

fn mouse_y(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.input.mouse_y as i64;
    ctx.push(value);
    Ok(())
}

fn mouse_wheel(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.input.mouse_wheel as i64;
    ctx.push(value);
    Ok(())
}

fn mouse_down(ctx: &mut Context) -> Result<(), ForthError> {
    let button = ctx.pop()? as i32;
    let down = ctx.input.mouse_down(button);
    ctx.push(if down { -1 } else { 0 });
    Ok(())
}

fn mouse_pressed(ctx: &mut Context) -> Result<(), ForthError> {
    let button = ctx.pop()? as i32;
    let pressed = ctx.input.mouse_pressed(button);
    ctx.push(if pressed { -1 } else { 0 });
    Ok(())
}

// Time

fn delta_time(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.time.dt_ms;
    ctx.push(value);
    Ok(())
}

fn elapsed_time(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.time.elapsed_ms;
    ctx.push(value);
    Ok(())
}

fn frame_counter(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.time.frame;
    ctx.push(value);
    Ok(())
}

// Memory

fn fetch(ctx: &mut Context) -> Result<(), ForthError> {
    let address = ctx.pop()?;
    let value = ctx.memory.get(address)?;
    ctx.push(value);
    Ok(())
}

fn store(ctx: &mut Context) -> Result<(), ForthError> {
    let address = ctx.pop()?;
    let value = ctx.pop()?;
    ctx.memory.set(address, value)?;
    Ok(())
}

fn add_store(ctx: &mut Context) -> Result<(), ForthError> {
    let address = ctx.pop()?;
    let value = ctx.pop()?;
    ctx.memory.add_to(address, value)?;
    Ok(())
}

fn here(ctx: &mut Context) -> Result<(), ForthError> {
    let address = ctx.memory.here() as i64;
    ctx.push(address);
    Ok(())
}

fn allot(ctx: &mut Context) -> Result<(), ForthError> {
    let count = ctx.pop()?;
    ctx.memory.allot(count)?;
    Ok(())
}

fn comma(ctx: &mut Context) -> Result<(), ForthError> {
    let value = ctx.pop()?;
    ctx.memory.push_cell(value);
    Ok(())
}

fn cells(ctx: &mut Context) -> Result<(), ForthError> {
    let count = ctx.pop()?;
    ctx.push(count.wrapping_mul(CELL_SIZE));
    Ok(())
}

fn cell_plus(ctx: &mut Context) -> Result<(), ForthError> {
    let address = ctx.pop()?;
    ctx.push(address.wrapping_add(CELL_SIZE));
    Ok(())
}
