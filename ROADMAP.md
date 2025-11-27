# 🐺 WolfLang v0.1 Development Roadmap

## 📅 Week 1: Brain & Logic (Completed: 80%)
The core of the language, reasoning capabilities, and memory management.
- [x] **Function Definition (`fn`):** Parser recognizes and registers functions.
- [x] **Function Call (`call`):** Executes functions like `add()`.
- [x] **Parameter Passing:** Passing values like `(x, y)` into functions.
- [x] **Scope & Recursion:** Self-calling functions support (Fibonacci Test ✅).
- [x] **`return` Statement:** Returning values from functions.
- [x] **Stress Testing:** Pushing the parser limits with edge cases.
- [x] **Embeddable Architecture:** Refactoring to `lib.rs` & `main.rs` structure. Now usable as a Rust crate! 📦

## 📅 Week 2: Memory & Data
Transitioning from single values to lists and data structures.
- [x] **Arrays:** Implementation of `let list x = [1, 2, 3]`.
- [x] **Indexing:** Reading data via `print x[0]`.
- [ ] **Methods:** Binding Rust vector methods like `push`, `pop`, `len`.
- [ ] **Mini Project:** A simple "Notepad" application using WolfLang.

## 📅 Week 3: Communication (Input/Output)
Enabling the language to interact with the outside world.
- [ ] **Native Functions:** Embedding Rust-based `input()` function.
- [x] **String Operations:** String concatenation (`"Hello " + name`).
- [ ] **Comments:** Ignoring lines starting with `//` in Lexer.
- [ ] **Game Demo:** "Number Guessing Game" (Testing Input and If/Else logic).

## 📅 Week 4: Packaging (Release)
Preparing the product for the showcase.
- [ ] **Documentation:** Comprehensive README.md (Setup & Usage Guide).
- [ ] **Visualization:** Adding ASCII Art logo and project description.
- [ ] **Code Cleanup:** Removing debug prints and optimizing code.
- [ ] **RELEASE v0.1:** Publishing the first version on GitHub 🚀.