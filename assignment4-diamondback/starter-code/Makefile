# Makefile for Diamondback Compiler

# Pattern rule to compile .snek files to .s assembly files
test/%.s: test/%.snek src/main.rs
	cargo run -- $< test/$*.s

# Pattern rule to assemble .s files and link into executables
test/%.run: test/%.s runtime/start.rs
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc -L runtime/ runtime/start.rs -o test/$*.run

# Clean build artifacts
clean:
	cargo clean
	rm -f test/*.s test/*.run runtime/*.o runtime/*.a

# Run all tests
test: test/37.run test/add.run test/negate.run test/complex.run test/factorial.run test/fibonacci.run test/mutual_recursion.run test/multi_arg.run test/nested_calls.run
	@echo "Running tests..."
	@./test/37.run
	@./test/add.run
	@./test/negate.run
	@./test/complex.run
	@./test/factorial.run
	@./test/fibonacci.run
	@./test/mutual_recursion.run
	@./test/multi_arg.run
	@./test/nested_calls.run

.PHONY: clean test
