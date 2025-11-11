#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <stdexcept>
#include <cstdint>
#include <iomanip>
#include <bitset>

#include "vtc_runtime.h"

using namespace VTC;

class VTCString {
private:
    char* ptr_;

public:
    explicit VTCString(char* ptr) : ptr_(ptr) {}

    ~VTCString() {
        if (ptr_) {
            runtime_free_string(ptr_);
        }
    }

    VTCString(const VTCString&) = delete;
    VTCString& operator=(const VTCString&) = delete;

    VTCString(VTCString&& other) noexcept : ptr_(other.ptr_) {
        other.ptr_ = nullptr;
    }

    const char* c_str() const { return ptr_ ? ptr_ : ""; }
    std::string str() const { return ptr_ ? std::string(ptr_) : std::string(); }
    operator bool() const { return ptr_ != nullptr; }
};

class VTCStringArray {
private:
    char** array_;
    size_t count_;

public:
    VTCStringArray() : array_(nullptr), count_(0) {}

    ~VTCStringArray() {
        if (array_) runtime_free_string_array(array_, count_);
    }

    VTCStringArray(const VTCStringArray&) = delete;
    VTCStringArray& operator=(const VTCStringArray&) = delete;

    char**& data() { return array_; }
    size_t& count() { return count_; }

    std::vector<std::string> to_vector() const {
        std::vector<std::string> result;
        result.reserve(count_);
        for (size_t i = 0; i < count_; ++i)
             result.emplace_back(array_[i]);
        return result;
    }
};

class VTCRuntime {
private:
    vtc_CRuntime runtime_;
public:
    VTCRuntime() : runtime_(runtime_new()) {
        if (!runtime_._0)
            throw std::runtime_error("Failed to create VTC runtime");
    }

    explicit VTCRuntime(const std::string& file_path) {
        runtime_ = runtime_from(file_path.c_str());
        if (!runtime_._0)
             throw std::runtime_error("Failed to create VTC runtime from file: " + file_path);
    }

    ~VTCRuntime() {
        if (runtime_._0)
            runtime_free(runtime_);
    }

    VTCRuntime(const VTCRuntime&) = delete;
    VTCRuntime& operator=(const VTCRuntime&) = delete;

    void load_file(const std::string& path) {
        if (runtime_load_file(runtime_, path.c_str()) != 0)
            throw std::runtime_error("Failed to load file: " + path);
    }

    void load_vtc(const std::string& input) {
        if (runtime_load_vtc(runtime_, input.c_str()) != 0)
            throw std::runtime_error("Failed to load VTC input");
    }

    std::vector<std::string> list_namespaces() const {
        VTCStringArray array;
        if (runtime_list_namespaces(runtime_, &array.data(), &array.count()) != 0)
            throw std::runtime_error("Failed to list namespaces");
        return array.to_vector();
    }

    std::vector<std::string> list_variables(const std::string& namespace_name) const {
        VTCStringArray array;
        if (runtime_list_variables(runtime_, namespace_name.c_str(),
                                   &array.data(), &array.count()) != 0)
            throw std::runtime_error("Failed to list variables in namespace: " + namespace_name);
        return array.to_vector();
    }

    std::string get_string(const std::string& namespace_name,
                          const std::string& variable) const {
        VTCString str(runtime_get_string(runtime_, namespace_name.c_str(),
                                        variable.c_str()));
        if (!str)
            throw std::runtime_error("Failed to get string variable: " +
                                   namespace_name + "." + variable);
        return str.str();
    }

    int64_t get_integer(const std::string& namespace_name,
                       const std::string& variable) const {
        int64_t value;
        if (runtime_get_integer(runtime_, namespace_name.c_str(),
                               variable.c_str(), &value) != 0)
            throw std::runtime_error("Failed to get integer variable: " +
                                   namespace_name + "." + variable);
        return value;
    }

    double get_float(const std::string& namespace_name,
                    const std::string& variable) const {
        double value;
        if (runtime_get_float(runtime_, namespace_name.c_str(),
                             variable.c_str(), &value) != 0)
            throw std::runtime_error("Failed to get float variable: " +
                                   namespace_name + "." + variable);
        return value;
    }

    bool get_boolean(const std::string& namespace_name,
                    const std::string& variable) const {
        bool value;
        if (runtime_get_boolean(runtime_, namespace_name.c_str(),
                               variable.c_str(), &value) != 0)
            throw std::runtime_error("Failed to get boolean variable: " +
                                   namespace_name + "." + variable);
        return value;
    }

    vtc_CRuntime raw() const { return runtime_; }
};

void print_separator(const std::string& title) {
    std::cout << "\n" << std::string(60, '=') << "\n";
    std::cout << "  " << title << "\n";
    std::cout << std::string(60, '=') << "\n";
}

void print_namespaces(const VTCRuntime& runtime) {
    std::cout << "\nAvailable Namespaces:\n";
    for (const auto& ns : runtime.list_namespaces()) {
        std::cout << "  - " << ns << "\n";
    }
}

void print_variables(const VTCRuntime& runtime, const std::string& namespace_name) {
    std::cout << "\nVariables in '@" << namespace_name << "':\n";
    auto variables = runtime.list_variables(namespace_name);
    std::cout << "  Total: " << variables.size() << " variables\n";
    for (const auto& var : variables) {
        std::cout << "  - $" << var << "\n";
    }
}

void demonstrate_integer_operations(const VTCRuntime& runtime) {
    print_separator("Integer Arithmetic Operations");

    const std::string ns = "intrinsics";

    struct IntOp {
        std::string var;
        std::string operation;
        std::string expected;
    };

    std::vector<IntOp> ops = {
        {"perform_add_int", "1 + 2", "3"},
        {"perform_sub_int", "5 - 3", "2"},
        {"perform_mul_int", "4 * 3", "12"},
        {"perform_div_int", "10 / 2", "5"},
        {"perform_mod_int", "10 % 3", "1"}
    };

    for (const auto& op : ops) {
        try {
            auto result = runtime.get_integer(ns, op.var);
            std::cout << std::left << std::setw(25) << ("$" + op.var)
                      << "= " << std::setw(10) << op.operation
                      << "= " << result
                      << " [OK]" << "\n";
        } catch (const std::exception& e) {
            std::cerr << "  [ERROR] Reading $" << op.var << ": " << e.what() << "\n";
        }
    }
}

void demonstrate_float_operations(const VTCRuntime& runtime) {
    print_separator("Floating-Point Arithmetic Operations");

    const std::string ns = "intrinsics";

    struct FloatOp {
        std::string var;
        std::string operation;
    };

    std::vector<FloatOp> ops = {
        {"perform_add_float", "1.5 + 2.7"},
        {"perform_sub_float", "5.5 - 3.2"},
        {"perform_mul_float", "4.0 * 3.5"},
        {"perform_div_float", "10.0 / 2.5"}
    };

    std::cout << std::fixed << std::setprecision(2);

    for (const auto& op : ops) {
        try {
            auto result = runtime.get_float(ns, op.var);
            std::cout << std::left << std::setw(25) << ("$" + op.var)
                      << "= " << std::setw(15) << op.operation
                      << "= " << result
                      << " [OK]" << "\n";
        } catch (const std::exception& e) {
            std::cerr << "  [ERROR] Reading $" << op.var << ": " << e.what() << "\n";
        }
    }
}

void demonstrate_type_conversions(const VTCRuntime& runtime) {
    print_separator("Type Conversion Operations");

    const std::string ns = "intrinsics";

    try {
        auto int_to_float = runtime.get_float(ns, "convert_int_to_float");
        std::cout << "$convert_int_to_float   = int_to_float(42)   = "
                  << std::fixed << std::setprecision(1) << int_to_float
                  << " [OK]\n";
    } catch (const std::exception& e) {
        std::cerr << "  [ERROR]: " << e.what() << "\n";
    }

    try {
        auto float_to_int = runtime.get_integer(ns, "convert_float_to_int");
        std::cout << "$convert_float_to_int   = float_to_int(3.14) = "
                  << float_to_int
                  << " [OK]\n";
    } catch (const std::exception& e) {
        std::cerr << "  [ERROR]: " << e.what() << "\n";
    }
}

void demonstrate_comparison_operations(const VTCRuntime& runtime) {
    print_separator("Comparison Operations");

    const std::string ns = "intrinsics";

    struct CompOp {
        std::string var;
        std::string operation;
    };

    std::vector<CompOp> ops = {
        {"compare_eq", "5 == 5"},
        {"compare_lt", "3 < 7"},
        {"compare_gt", "8 > 2"}
    };

    for (const auto& op : ops) {
        try {
            auto result = runtime.get_boolean(ns, op.var);
            std::cout << std::left << std::setw(20) << ("$" + op.var)
                      << "= " << std::setw(10) << op.operation
                      << "= " << (result ? "true" : "false")
                      << " [OK]" << "\n";
        } catch (const std::exception& e) {
            std::cerr << "  [ERROR] Reading $" << op.var << ": " << e.what() << "\n";
        }
    }
}

void demonstrate_bitwise_operations(const VTCRuntime& runtime) {
    print_separator("Bitwise Operations");

    const std::string ns = "intrinsics";

    struct BitwiseOp {
        std::string var;
        std::string operation;
    };

    std::vector<BitwiseOp> ops = {
        {"bitwise_and", "5 & 3"},
        {"bitwise_or",  "5 | 3"},
        {"bitwise_xor", "5 ^ 3"},
        {"bitwise_not", "~5"}
    };

    for (const auto& op : ops) {
        try {
            auto result = runtime.get_integer(ns, op.var);
            std::cout << std::left << std::setw(20) << ("$" + op.var)
                      << "= " << std::setw(10) << op.operation
                      << "= " << std::setw(5) << result
                      << " (0b" << std::bitset<8>(result) << ")"
                      << "  " << "\n";
        } catch (const std::exception& e) {
            std::cerr << "   [ERROR] Reading $" << op.var << ": " << e.what() << "\n";
        }
    }
}

void print_summary(const VTCRuntime& runtime) {
    print_separator("Summary");

    const std::string ns = "intrinsics";
    auto variables = runtime.list_variables(ns);

    std::cout << "Successfully loaded VTC file with:\n";
    std::cout << "  * " << runtime.list_namespaces().size() << " namespace(s)\n";
    std::cout << "  * " << variables.size() << " computed variable(s)\n";
    std::cout << "\nAll intrinsic functions executed successfully!\n";
}

int main(int argc, char** argv) {
    try {
        std::cout << "\n";
        std::cout << "+============================================================+\n";
        std::cout << "|         VTC Runtime - C++ FFI Example                      |\n";
        std::cout << "|         Demonstrating Intrinsic Functions                  |\n";
        std::cout << "+============================================================+\n";

        VTCRuntime runtime;

        const std::string file_path = (argc > 1) ? argv[1] : "./samples/intrinsics.vtc";
        std::cout << "\nLoading: " << file_path << "\n";

        runtime.load_file(file_path);
        std::cout << "File loaded successfully!\n";

        print_namespaces(runtime);
        print_variables(runtime, "intrinsics");

        demonstrate_integer_operations(runtime);
        demonstrate_float_operations(runtime);
        demonstrate_type_conversions(runtime);
        demonstrate_comparison_operations(runtime);
        demonstrate_bitwise_operations(runtime);

        print_summary(runtime);
        std::cout << "\n" << std::string(60, '=') << "\n";
        std::cout << "Demo completed successfully!\n\n";

        return 0;

    } catch (const std::exception& e) {
        std::cerr << "\nFatal error: " << e.what() << "\n\n";
        return 1;
    } catch (...) {
        std::cerr << "\nUnknown fatal error occurred\n\n";
        return 1;
    }
}