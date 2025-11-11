CXX = g++
CXXFLAGS = -std=c++17 -Wall -Werror -I./c
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

all: example_program comprehensive_example

rust_lib:
	cargo build --release

example_program: example_program.o rust_lib
	$(CXX) example_program.o $(LDFLAGS) -o $@

example_program.o: ./c/ExampleCxx.cxx
	$(CXX) $(CXXFLAGS) -c $< -o $@

comprehensive_example: comprehensive_example.o rust_lib
	$(CXX) comprehensive_example.o $(LDFLAGS) -o $@

comprehensive_example.o: ./c/ComprehensiveExample.cxx
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f example_program example_program.o comprehensive_example comprehensive_example.o
	cargo clean