# Default target runs all checks
all_checks: clippy test check fmt

# Run cargo clippy with additional flags for stricter checking
clippy:
	@echo "Running cargo clippy..."
	cargo clippy --all-features -- -D warnings

# Run cargo test
test:
	@echo "Running cargo test..."
	cargo test --all-features

# Run cargo check
check:
	@echo "Running cargo check..."
	cargo check --all-features

fmt:
	@echo "Formatting code..."
	cargo fmt --all

build_commit_diff:
	@echo "Building recent commits history..."
	git log -3 --pretty=format:"%B" | sed 's/"/\\"/g' > commits.txt
	@echo "Building diff..."
	git diff --cached --diff-algorithm=minimal > diff.txt

# Clean the project
clean:
	@echo "Cleaning project..."
	rm commits.txt diff.txt
	cargo clean
