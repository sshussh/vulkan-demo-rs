# Vulkan Triangle Demo Makefile
# ============================

# Project configuration
SHADER_DIR := shaders
SRC_SHADER_DIR := src/shaders
CARGO := cargo
GLSLC := glslc

# Default target
.PHONY: all
all: run

# Run the application
.PHONY: run
run: shaders
	$(CARGO) run --release

# Build the project
.PHONY: build
build: shaders
	$(CARGO) build --release

# Compile shaders
.PHONY: shaders
shaders: $(SRC_SHADER_DIR)/vert.spv $(SRC_SHADER_DIR)/frag.spv

$(SRC_SHADER_DIR)/vert.spv: $(SHADER_DIR)/shader.vert | $(SRC_SHADER_DIR)
	$(GLSLC) $< -o $@

$(SRC_SHADER_DIR)/frag.spv: $(SHADER_DIR)/shader.frag | $(SRC_SHADER_DIR)
	$(GLSLC) $< -o $@

$(SRC_SHADER_DIR):
	mkdir -p $(SRC_SHADER_DIR)

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean
	rm -f $(SRC_SHADER_DIR)/*.spv

# Test the project
.PHONY: test
test:
	$(CARGO) test

# Check code
.PHONY: check
check:
	$(CARGO) check
