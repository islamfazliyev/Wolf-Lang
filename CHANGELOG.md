# Changelog

## [0.1.5] - 2026-05-29
### Added
- `struct-impl` structures added
- `import "example.wolf" as ex` import system for wolflang files
- Line based error logs

### Changed
- language seperated from parser to interpreter and ast

## [0.1.1] - 2025-12-02
### Added
- `list<list<Type>>` support for 2D arrays (and more).
- Native `clear()` function for terminal clearing.
- Example `map_game.wolf` script.

### Changed
- Syntax updated to Declaration with Type Annotation Before Initialization: `let name: type = value`.

### Fixed
- fixed reassignment type mismatch not giving error
