<p align="center">
  <img src="assets/wolflang.svg" alt="WolfLang Logo" width="200">
</p>

# 🐺 WolfLang

> **A Lua-inspired, embeddable, statically-typed scripting language written in Rust.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Made With Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-v0.1.0--alpha-blue)]()
[![Docs](https://img.shields.io/badge/Docs-docs-orange)](https://islamfazliyev.github.io/wolflang-docs/)

WolfLang is designed for **scripting**, **quick prototyping**, and **embedding** into larger applications such as game engines. It combines the clean syntax of Lua/Ruby with static typing and a Rust-powered runtime.

---

## ✨ Features (v0.1.5)

- 📦 **Embeddable Architecture** — Use as a Rust crate; share data between Rust and WolfLang with a simple API.
- 🔒 **Static Typing** — Types are checked at assignment time: `int`, `float`, `bool`, `string`, `list<T>`.
- 🔄 **Recursion Support** — Full support for recursive function calls.
- 🎒 **Dynamic Lists** — Create, index, and mutate lists with built-in `push`, `pop`, and `len` methods.
- 🏗️ **Structs & Impl Blocks** — Define custom data types and attach methods to them.
- 📂 **Module Imports** — Import other `.wolf` files as namespaced modules.
- 🔌 **Rust Interop** — Call Rust functions from WolfLang (`push_fn`) and call WolfLang functions from Rust (`get_fn`).
- 💬 **Native I/O** — Built-in `input()` and `clear()` functions.

---

## 🚀 Getting Started

### CLI Usage

```bash
# Clone the repository
git clone https://github.com/islamfazliyev/Wolf-Lang.git
cd Wolf-Lang

# Build in release mode
cargo build --release

# Run a script
./target/release/wolflang --file examples/text_game.wolf
```

### Embedding in a Rust Project

Add WolfLang to your `Cargo.toml`:

```toml
[dependencies]
wolflang = { git = "https://github.com/islamfazliyev/Wolf-Lang.git" }
```

Basic usage:

```rust
use wolflang::WolfEngine;

fn main() {
    let mut engine = WolfEngine::new();

    // Push data into WolfLang
    engine.push_int("player_hp", 100);

    let script = r#"
        print "Current HP: ", player_hp
        player_hp = player_hp - 10
    "#;

    engine.run(script).unwrap();

    // Read data back from WolfLang
    let new_hp = engine.get_int("player_hp").unwrap();
    println!("HP from Rust: {}", new_hp); // 90
}
```

---

## 📖 Syntax Tour

### Variables

Variables are declared with `let`, a name, a type annotation, and a value.

```wolf
let name: string = "WolfLang"
let version: int = 1
let pi: float = 3.14
let is_fast: bool = true
```

Re-assignment (without `let`) is also supported, and the type is enforced:

```wolf
version = 2         # OK
version = "two"     # Runtime error: type mismatch
```

### Comments

```wolf
# This is a single-line comment
```

### Lists

```wolf
let inventory: list<string> = ["Sword", "Shield"]

inventory.push("Potion")
print inventory[0]          # Sword
print inventory.len()       # 3

let item: string = inventory.pop()
print item                  # Potion
```

Multidimensional lists and index assignment are also supported:

```wolf
let grid: list<list<int>> = [[10, 20],[90, 99]]
grid[0][0] = 99
print grid[0][0]               # 99
```

### Control Flow

```wolf
if version >= 1
    print "Ready for release!"
end

if version < 1
    print "Still in beta..."
else
    print "Stable!"
end
```

### Loops

```wolf
# While loop
let i: int = 0
while i < 5
    print i
    i = i + 1
end

# For loop (range is exclusive)
for int i = 0 range 10
    print i
end
```

### Functions & Recursion

```wolf
fn greet(name: string)
    print "Hello, " + name
end

greet("World")

fn fibonacci(n: int)
    if n <= 1
        return n
    end
    return fibonacci(n - 1) + fibonacci(n - 2)
end

print fibonacci(10)   # 55
```

### Structs & Impl

Define a struct and attach methods with `impl`:

```wolf
struct Point
    x: int
    y: int
end

impl Point
    fn get_x()
        return self.x
    end
    fn get_y()
        return self.y
    end
end

let p: Point = Point(10, 20)
print p.get_x()   # 10
print p.get_y()   # 20

# Field access and mutation
print p.x         # 10
p.x = 99
print p.x         # 99
```

### Module Imports

Split your code into multiple files and import them as namespaced modules:

```wolf
# math.wolf
fn add(a: int, b: int)
    return a + b
end
```

```wolf
# main.wolf
import "math.wolf" as math

let result: int = math.add(5, 3)
print result   # 8
```

Struct definitions and impl blocks from imported modules are also namespaced:

```wolf
import "geometry.wolf" as geo

let p: geo::Point = geo::Point(1, 2)
```

### Built-in I/O

```wolf
let name: string = input("Enter your name: ")
print "Hello, " + name
clear()
```

---

## 🔌 Embedding API Reference

### Pushing values into WolfLang

```rust
engine.push_int("x", 42);
engine.push_float("speed", 1.5);
engine.push_str("tag", "player");
engine.push_bool("alive", true);
engine.push_list("items", vec![Token::Integer(1), Token::Integer(2)]);
```

### Reading values from WolfLang

```rust
let x    = engine.get_int("x");       // Option<i64>
let spd  = engine.get_float("speed"); // Option<f64>
let tag  = engine.get_str("tag");     // Option<String>
let alive= engine.get_bool("alive");  // Option<bool>
let list = engine.get_list("items");  // Option<Vec<Token>>
```

### Registering Rust functions

```rust
engine.push_fn("add", |args| {
    if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (args.get(0), args.get(1)) {
        Token::Integer(a + b)
    } else {
        Token::Unknown
    }
});

engine.run(r#"
    let result: int = add(10, 20)
    print result    # 30
"#).unwrap();
```

### Calling WolfLang functions from Rust

```rust
engine.run(r#"
    fn multiply(x: int, y: int)
        return x * y
    end
"#).unwrap();

let result = engine.get_fn("multiply", vec![Token::Integer(6), Token::Integer(7)]);
assert_eq!(result, Some(Token::Integer(42)));
```

---

## 🤝 Contributing

This is an early-stage project developed by a solo developer. Issues, pull requests, and feedback are highly welcome!

**Developed by Islam Şahin**
