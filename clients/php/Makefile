# Makefile for MerkleKV PHP Client

.PHONY: help install test test-unit test-integration test-performance test-coverage clean lint format docs examples

# Default target
help:
	@echo "MerkleKV PHP Client - Available Commands:"
	@echo ""
	@echo "  install           Install dependencies via Composer"
	@echo "  test              Run all tests (unit + integration)"
	@echo "  test-unit         Run unit tests only"
	@echo "  test-integration  Run integration tests (requires server)"
	@echo "  test-performance  Run performance tests (requires server)"
	@echo "  test-coverage     Run tests with coverage report"
	@echo "  lint              Run PHP CS Fixer (code style)"
	@echo "  format            Auto-fix code style issues"
	@echo "  docs              Generate API documentation"
	@echo "  examples          Run example scripts"
	@echo "  clean             Clean generated files"
	@echo ""
	@echo "Requirements:"
	@echo "  - PHP 7.4+"
	@echo "  - Composer"
	@echo "  - MerkleKV server for integration tests"

# Install dependencies
install:
	@echo "Installing Composer dependencies..."
	composer install

# Run all tests
test: test-unit test-integration

# Run unit tests only
test-unit:
	@echo "Running unit tests..."
	vendor/bin/phpunit --testsuite="Unit Tests"

# Run integration tests (requires server)
test-integration:
	@echo "Running integration tests..."
	@echo "Note: Requires MerkleKV server on 127.0.0.1:7379"
	vendor/bin/phpunit --testsuite="Integration Tests" --group=integration

# Run performance tests
test-performance:
	@echo "Running performance tests..."
	@echo "Note: Requires MerkleKV server on 127.0.0.1:7379"
	vendor/bin/phpunit --group=performance

# Run tests with coverage
test-coverage:
	@echo "Running tests with coverage..."
	vendor/bin/phpunit --coverage-html coverage/ --coverage-text

# Check code style
lint:
	@echo "Checking code style..."
	@if [ -f vendor/bin/php-cs-fixer ]; then \
		vendor/bin/php-cs-fixer fix --dry-run --diff; \
	else \
		echo "PHP CS Fixer not installed. Run: composer require --dev friendsofphp/php-cs-fixer"; \
	fi

# Auto-fix code style
format:
	@echo "Fixing code style..."
	@if [ -f vendor/bin/php-cs-fixer ]; then \
		vendor/bin/php-cs-fixer fix; \
	else \
		echo "PHP CS Fixer not installed. Run: composer require --dev friendsofphp/php-cs-fixer"; \
	fi

# Generate documentation
docs:
	@echo "Generating API documentation..."
	@if [ -f vendor/bin/phpdoc ]; then \
		vendor/bin/phpdoc -d src/ -t docs/; \
	else \
		echo "phpDocumentor not installed. Run: composer require --dev phpdocumentor/phpdocumentor"; \
	fi

# Run examples
examples:
	@echo "Running basic example..."
	php examples/basic.php
	@echo ""
	@echo "Running performance example..."
	php examples/performance.php

# Clean generated files
clean:
	@echo "Cleaning generated files..."
	rm -rf coverage/
	rm -rf docs/
	rm -rf vendor/
	rm -f composer.lock

# Development server for testing
server:
	@echo "Starting MerkleKV server for testing..."
	@echo "Note: This assumes MerkleKV binary is in PATH"
	@echo "Starting server on 127.0.0.1:7379..."
	merkle_kv --host 127.0.0.1 --port 7379

# Quick validation
validate: install test-unit
	@echo "✅ PHP client validation complete!"

# CI pipeline
ci: install test test-coverage
	@echo "✅ CI pipeline complete!"
