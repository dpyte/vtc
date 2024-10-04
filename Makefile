# Makefile

CC = gcc
CFLAGS = -Wall -Werror -I./c
LDFLAGS = -L./target/release -lvtc

# Detect the operating system
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -Wl,-rpath,./target/release
endif
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -Wl,-rpath,@executable_path/../target/release
endif

.PHONY: all clean rust_lib

all: example_program

rust_lib:
	cargo build --release

example_program: example_program.o rust_lib
	$(CC) example_program.o $(LDFLAGS) -o $@

example_program.o: ./c/example_program.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f example_program example_program.o
	cargo clean