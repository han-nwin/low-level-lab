
# Variables
CC = gcc
CFLAGS = -Iinclude -Wall -Wextra -g
SRC_DIR = src
OBJ_DIR = build
BIN_DIR = bin
TARGET = $(BIN_DIR)/c_program

# File lists
C_SOURCES = $(wildcard $(SRC_DIR)/*.c)
C_OBJECTS = $(C_SOURCES:$(SRC_DIR)/%.c=$(OBJ_DIR)/%.o)

# Default target
all: $(TARGET)

# Link object files to create the final binary
$(TARGET): $(C_OBJECTS)
	@mkdir -p $(BIN_DIR)
	$(CC) $(C_OBJECTS) -o $@

# Compile C source files
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# Clean up build artifacts
clean:
	rm -rf $(OBJ_DIR) $(BIN_DIR)

# Run the program
run: all
	./$(TARGET)

# Phony targets
.PHONY: all clean run
