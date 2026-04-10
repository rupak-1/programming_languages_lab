# Diamondback Compiler Notes

This compiler extends Cobra with first-order functions and function calls.

## Calling Convention Used

- **Function labels**: each definition `(fun (name args...) body)` compiles to `fun_name`.
- **Caller responsibilities**:
  - evaluate call arguments into temporary frame slots
  - push arguments **right-to-left**
  - perform one-word alignment padding before call when needed
  - `call fun_name`
  - clean up all pushed words (arguments + optional padding)
- **Callee responsibilities**:
  - prologue: `push rbp`, `mov rbp, rsp`
  - allocate local frame with `sub rsp, N` (16-byte aligned)
  - evaluate body
  - epilogue: `mov rsp, rbp`, `pop rbp`, `ret`
- **Parameter locations**:
  - first parameter at `[rbp + 16]`
  - second at `[rbp + 24]`
  - etc.
- **Local/temporary locations**:
  - first local slot at `[rbp - 8]`
  - then `[rbp - 16]`, ...

## Error Checks Added for Functions

- wrong arity at call site
- call to undefined function
- shadowing function parameters with `let` bindings

## Built-in Print

`print` is implemented as a unary operator in the language and compiles to a runtime call:

- `call snek_print`
- runtime prints tagged values as numbers/bools and returns the original value

## Interesting Example Programs

### Factorial

```scheme
((fun (fact n)
   (if (= n 1)
       1
       (* n (fact (sub1 n)))))
 (fact 5))
```

### Fibonacci

```scheme
((fun (fib n)
   (if (<= n 1)
       n
       (+ (fib (sub1 n))
          (fib (sub1 (sub1 n))))))
 (fib 8))
```

### Mutual Recursion

```scheme
((fun (is_even n)
   (if (= n 0) true (is_odd (sub1 n))))
 (fun (is_odd n)
   (if (= n 0) false (is_even (sub1 n))))
 (is_even 8))
```
