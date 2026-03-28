# Cobra value tagging (Assignment 3)

This compiler uses the **least significant bit** as the runtime type tag.

## Representation

| Kind    | LSB | Pattern (binary) | Decimal (example) |
|---------|-----|------------------|-------------------|
| Number  | 0   | `value << 1`     | Integer `n` is encoded as `2n` (e.g. `5` → `10`) |
| Boolean | 1   | fixed constants  | `false` → `1`, `true` → `3` |

- **Numbers**: shifted left by one so the LSB is always `0`. Arithmetic on two tagged numbers can use `add` / `sub` directly on the encoded values when the operation corresponds to the same operation on the underlying integers (after overflow checks where required).
- **Booleans**: only the values `1` (`false`) and `3` (`true`) are produced; both have LSB `1`.

## Decoding

- **Number**: `decoded_int = tagged >> 1` (signed arithmetic as appropriate in assembly).
- **Boolean**: compare to `1` (false) or `3` (true); other odd values are not produced by this compiler for booleans.

## Runtime output

The Rust runtime (`runtime/start.rs`) prints decoded values: numbers as decimal integers, booleans as `true` or `false`.

## Errors

- **Invalid argument** (`snek_error(1)`): type mismatch (e.g. `+` on non-numbers, `=` on mixed types, comparisons on non-numbers).
- **Overflow** (`snek_error(2)`): signed overflow from arithmetic (`add`, `sub`, `imul`, `neg`, etc.).
