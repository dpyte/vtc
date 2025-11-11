#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <stdexcept>
#include <cstdint>
#include <iomanip>
#include <bitset>
#include <algorithm>

#include "vtc_runtime.h"

using namespace VTC;

// RAII wrapper for VTC strings
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

// RAII wrapper for VTC string arrays
class VTCStringArray {
private:
    char** array_;
    size_t count_;

public:
    VTCStringArray() : array_(nullptr), count_(0) {}

    ~VTCStringArray() {
        if (array_) {
            runtime_free_string_array(array_, count_);
        }
    }

    VTCStringArray(const VTCStringArray&) = delete;
    VTCStringArray& operator=(const VTCStringArray&) = delete;

    char**& data() { return array_; }
    size_t& count() { return count_; }

    std::vector<std::string> to_vector() const {
        std::vector<std::string> result;
        result.reserve(count_);
        for (size_t i = 0; i < count_; ++i) {
            result.emplace_back(array_[i]);
        }
        return result;
    }
};

// Main RAII wrapper for VTC Runtime
class VTCRuntime {
private:
    vtc_CRuntime runtime_;

public:
    VTCRuntime() : runtime_(runtime_new()) {
        if (!runtime_._0) {
            throw std::runtime_error("Failed to create VTC runtime");
        }
    }

    explicit VTCRuntime(const std::string& file_path) {
        runtime_ = runtime_from(file_path.c_str());
        if (!runtime_._0) {
            throw std::runtime_error("Failed to create VTC runtime from file: " + file_path);
        }
    }

    ~VTCRuntime() {
        if (runtime_._0) {
            runtime_free(runtime_);
        }
    }

    VTCRuntime(const VTCRuntime&) = delete;
    VTCRuntime& operator=(const VTCRuntime&) = delete;

    void load_file(const std::string& path) {
        if (runtime_load_file(runtime_, path.c_str()) != 0) {
            throw std::runtime_error("Failed to load file: " + path);
        }
    }

    void load_vtc(const std::string& input) {
        if (runtime_load_vtc(runtime_, input.c_str()) != 0) {
            throw std::runtime_error("Failed to load VTC input");
        }
    }

    std::vector<std::string> list_namespaces() const {
        VTCStringArray array;
        if (runtime_list_namespaces(runtime_, &array.data(), &array.count()) != 0) {
            throw std::runtime_error("Failed to list namespaces");
        }
        return array.to_vector();
    }

    std::vector<std::string> list_variables(const std::string& namespace_name) const {
        VTCStringArray array;
        if (runtime_list_variables(runtime_, namespace_name.c_str(),
                                   &array.data(), &array.count()) != 0) {
            throw std::runtime_error("Failed to list variables in namespace: " + namespace_name);
        }
        return array.to_vector();
    }

    std::string get_string(const std::string& namespace_name,
                          const std::string& variable) const {
        VTCString str(runtime_get_string(runtime_, namespace_name.c_str(),
                                        variable.c_str()));
        if (!str) {
            throw std::runtime_error("Failed to get string variable: " +
                                   namespace_name + "." + variable);
        }
        return str.str();
    }

    int64_t get_integer(const std::string& namespace_name,
                       const std::string& variable) const {
        int64_t value;
        if (runtime_get_integer(runtime_, namespace_name.c_str(),
                               variable.c_str(), &value) != 0) {
            throw std::runtime_error("Failed to get integer variable: " +
                                   namespace_name + "." + variable);
        }
        return value;
    }

    double get_float(const std::string& namespace_name,
                    const std::string& variable) const {
        double value;
        if (runtime_get_float(runtime_, namespace_name.c_str(),
                             variable.c_str(), &value) != 0) {
            throw std::runtime_error("Failed to get float variable: " +
                                   namespace_name + "." + variable);
        }
        return value;
    }

    bool get_boolean(const std::string& namespace_name,
                    const std::string& variable) const {
        bool value;
        if (runtime_get_boolean(runtime_, namespace_name.c_str(),
                               variable.c_str(), &value) != 0) {
            throw std::runtime_error("Failed to get boolean variable: " +
                                   namespace_name + "." + variable);
        }
        return value;
    }

    vtc_CRuntime raw() const { return runtime_; }
};

// Helper functions
void print_separator(const std::string& title) {
    std::cout << "\n" << std::string(70, '=') << "\n";
    std::cout << "  " << title << "\n";
    std::cout << std::string(70, '=') << "\n";
}

void print_header() {
    std::cout << "\n";
    std::cout << "+====================================================================+\n";
    std::cout << "|         VTC Runtime - Comprehensive Configuration Demo            |\n";
    std::cout << "|         Repository: github.com/dpyte/vtc.git                       |\n";
    std::cout << "+====================================================================+\n";
}

void print_namespaces(const VTCRuntime& runtime) {
    auto namespaces = runtime.list_namespaces();
    std::cout << "\nTotal Namespaces: " << namespaces.size() << "\n";
    std::cout << "Available Namespaces:\n";
    for (const auto& ns : namespaces) {
        auto vars = runtime.list_variables(ns);
        std::cout << "  - @" << std::left << std::setw(25) << ns
                  << " (" << std::setw(3) << vars.size() << " variables)\n";
    }
}

void demonstrate_system_config(const VTCRuntime& runtime) {
    print_separator("System Configuration");
    const std::string ns = "system_config";

    try {
        auto max_conn = runtime.get_integer(ns, "max_connections");
        auto timeout = runtime.get_integer(ns, "timeout_ms");
        auto retry = runtime.get_integer(ns, "retry_attempts");
        auto debug = runtime.get_boolean(ns, "debug_mode");
        auto version = runtime.get_string(ns, "config_version");

        std::cout << "Max Connections:    " << max_conn << "\n";
        std::cout << "Timeout (ms):       " << timeout << "\n";
        std::cout << "Retry Attempts:     " << retry << "\n";
        std::cout << "Debug Mode:         " << (debug ? "Enabled" : "Disabled") << "\n";
        std::cout << "Config Version:     " << version << "\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_user_data(const VTCRuntime& runtime) {
    print_separator("User Data Analytics");
    const std::string ns = "user_data";

    try {
        auto total = runtime.get_integer(ns, "total_users");
        auto active = runtime.get_integer(ns, "active_users");
        auto session_time = runtime.get_float(ns, "average_session_time");

        double active_percent = (static_cast<double>(active) / total) * 100.0;

        std::cout << "Total Users:        " << total << "\n";
        std::cout << "Active Users:       " << active
                  << " (" << std::fixed << std::setprecision(2)
                  << active_percent << "%)\n";
        std::cout << "Avg Session Time:   " << std::fixed << std::setprecision(1)
                  << session_time << " seconds\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_metrics(const VTCRuntime& runtime) {
    print_separator("Performance Metrics");
    const std::string ns = "metrics";

    try {
        std::cout << "Server Load Analysis:\n";
        std::cout << "  Load Values: ";

        // Note: Arrays would need additional FFI support
        // For now, showing scalar values
        std::cout << "[Array data - requires array FFI support]\n";

    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_security(const VTCRuntime& runtime) {
    print_separator("Security Configuration");
    const std::string ns = "security";

    try {
        auto min_len = runtime.get_integer(ns, "password_min_length");
        auto require_special = runtime.get_boolean(ns, "password_require_special");
        auto require_numbers = runtime.get_boolean(ns, "password_require_numbers");
        auto require_upper = runtime.get_boolean(ns, "password_require_uppercase");
        auto max_attempts = runtime.get_integer(ns, "max_login_attempts");
        auto lockout = runtime.get_integer(ns, "lockout_duration_minutes");
        auto two_factor = runtime.get_boolean(ns, "two_factor_auth_enabled");

        std::cout << "Password Policy:\n";
        std::cout << "  Min Length:         " << min_len << " characters\n";
        std::cout << "  Special Chars:      " << (require_special ? "Required" : "Optional") << "\n";
        std::cout << "  Numbers:            " << (require_numbers ? "Required" : "Optional") << "\n";
        std::cout << "  Uppercase:          " << (require_upper ? "Required" : "Optional") << "\n";
        std::cout << "\nLogin Security:\n";
        std::cout << "  Max Attempts:       " << max_attempts << "\n";
        std::cout << "  Lockout Duration:   " << lockout << " minutes\n";
        std::cout << "  Two-Factor Auth:    " << (two_factor ? "Enabled" : "Disabled") << "\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_performance(const VTCRuntime& runtime) {
    print_separator("Performance Tuning");
    const std::string ns = "performance";

    try {
        auto cache_size = runtime.get_integer(ns, "cache_size_mb");
        auto max_threads = runtime.get_integer(ns, "max_threads");
        auto pool_size = runtime.get_integer(ns, "connection_pool_size");
        auto query_timeout = runtime.get_integer(ns, "query_timeout_ms");
        auto rebuild_interval = runtime.get_integer(ns, "index_rebuild_interval_hours");

        std::cout << "Resource Allocation:\n";
        std::cout << "  Cache Size:         " << cache_size << " MB\n";
        std::cout << "  Max Threads:        " << max_threads << "\n";
        std::cout << "  Connection Pool:    " << pool_size << " connections\n";
        std::cout << "\nTiming Configuration:\n";
        std::cout << "  Query Timeout:      " << query_timeout << " ms\n";
        std::cout << "  Index Rebuild:      " << rebuild_interval << " hours\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_feature_flags(const VTCRuntime& runtime) {
    print_separator("Feature Flags");
    const std::string ns = "feature_flags";

    try {
        auto new_ui = runtime.get_boolean(ns, "new_ui_enabled");
        auto rollout = runtime.get_float(ns, "rollout_percentage");
        auto max_beta = runtime.get_integer(ns, "max_beta_users");

        std::cout << "Feature Rollout Status:\n";
        std::cout << "  New UI:             " << (new_ui ? "Enabled" : "Disabled") << "\n";
        std::cout << "  Rollout Progress:   " << std::fixed << std::setprecision(1)
                  << (rollout * 100.0) << "%\n";
        std::cout << "  Max Beta Users:     " << max_beta << "\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_localization(const VTCRuntime& runtime) {
    print_separator("Localization Configuration");
    const std::string ns = "localization";

    try {
        auto default_lang = runtime.get_string(ns, "default_language");

        std::cout << "Language Configuration:\n";
        std::cout << "  Default Language:   " << default_lang << "\n";
        std::cout << "  Supported Languages: [Array data - requires array FFI support]\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_calculations(const VTCRuntime& runtime) {
    print_separator("Calculations and Expressions");
    const std::string ns = "calculations";

    try {
        auto simple_add = runtime.get_integer(ns, "simple_addition");
        auto nested_calc = runtime.get_integer(ns, "nested_calculation");

        std::cout << "Expression Results:\n";
        std::cout << "  Simple Addition:    10 + 20 = " << simple_add << "\n";
        std::cout << "  Nested Calculation: (5+10) * (50-25) = " << nested_calc << "\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_bitwise_operations(const VTCRuntime& runtime) {
    print_separator("Bitwise Operations");
    const std::string ns = "bitwise_operations";

    try {
        auto flags = runtime.get_integer(ns, "flags");
        auto mask = runtime.get_integer(ns, "mask");
        auto bit_and = runtime.get_integer(ns, "bitwise_and");
        auto bit_or = runtime.get_integer(ns, "bitwise_or");
        auto bit_xor = runtime.get_integer(ns, "bitwise_xor");
        auto bit_not = runtime.get_integer(ns, "bitwise_not");

        std::cout << "Input Values:\n";
        std::cout << "  flags = " << std::setw(3) << flags
                  << " (0b" << std::bitset<8>(flags) << ")\n";
        std::cout << "  mask  = " << std::setw(3) << mask
                  << " (0b" << std::bitset<8>(mask) << ")\n";
        std::cout << "\nOperations:\n";
        std::cout << "  AND   = " << std::setw(3) << bit_and
                  << " (0b" << std::bitset<8>(bit_and) << ")\n";
        std::cout << "  OR    = " << std::setw(3) << bit_or
                  << " (0b" << std::bitset<8>(bit_or) << ")\n";
        std::cout << "  XOR   = " << std::setw(3) << bit_xor
                  << " (0b" << std::bitset<8>(bit_xor) << ")\n";
        std::cout << "  NOT   = " << std::setw(3) << bit_not
                  << " (0b" << std::bitset<8>(bit_not & 0xFF) << ")\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_string_operations(const VTCRuntime& runtime) {
    print_separator("String Operations");
    const std::string ns = "string_operations";

    try {
        auto base = runtime.get_string(ns, "base_string");
        auto upper = runtime.get_string(ns, "uppercase");
        auto lower = runtime.get_string(ns, "lowercase");
        auto substr = runtime.get_string(ns, "substring");
        auto concat = runtime.get_string(ns, "concat");
        auto replace = runtime.get_string(ns, "replace");

        std::cout << "Base String:        \"" << base << "\"\n";
        std::cout << "Uppercase:          \"" << upper << "\"\n";
        std::cout << "Lowercase:          \"" << lower << "\"\n";
        std::cout << "Substring [0:5]:    \"" << substr << "\"\n";
        std::cout << "Concatenated:       \"" << concat << "\"\n";
        std::cout << "Replaced:           \"" << replace << "\"\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_advanced_operations(const VTCRuntime& runtime) {
    print_separator("Advanced Operations");
    const std::string ns = "advanced_operations";

    try {
        auto encoded = runtime.get_string(ns, "base64_encoded");
        auto decoded = runtime.get_string(ns, "base64_decoded");
        auto hash = runtime.get_string(ns, "hash");

        std::cout << "Base64 Encoding:\n";
        std::cout << "  Original:   \"Secret message\"\n";
        std::cout << "  Encoded:    \"" << encoded << "\"\n";
        std::cout << "  Decoded:    \"" << decoded << "\"\n";
        std::cout << "\nHashing:\n";
        std::cout << "  SHA-256:    \"" << hash << "\"\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void demonstrate_advanced_math(const VTCRuntime& runtime) {
    print_separator("Advanced Mathematical Operations");
    const std::string ns = "advanced_math";

    try {
        auto compound = runtime.get_float(ns, "compound_interest");
        auto weighted = runtime.get_float(ns, "weighted_average");
        auto mixed = runtime.get_float(ns, "mixed_operations");
        auto temp_conv = runtime.get_float(ns, "temperature_conversion");
        auto circle = runtime.get_float(ns, "circle_calculations");

        std::cout << std::fixed << std::setprecision(2);
        std::cout << "Financial Calculations:\n";
        std::cout << "  Compound Interest:  $" << compound << "\n";
        std::cout << "\nStatistical Calculations:\n";
        std::cout << "  Weighted Average:   " << weighted << "\n";
        std::cout << "\nType Mixing:\n";
        std::cout << "  Mixed Operations:   " << mixed << "\n";
        std::cout << "\nUnit Conversions:\n";
        std::cout << "  Temp (25C to F):    " << temp_conv << " F\n";
        std::cout << "\nGeometric Calculations:\n";
        std::cout << "  Circle Area (r=10): " << circle << " sq units\n";
    } catch (const std::exception& e) {
        std::cerr << "[ERROR] " << e.what() << "\n";
    }
}

void print_summary(const VTCRuntime& runtime) {
    print_separator("Summary");

    auto namespaces = runtime.list_namespaces();
    size_t total_vars = 0;

    for (const auto& ns : namespaces) {
        total_vars += runtime.list_variables(ns).size();
    }

    std::cout << "Configuration loaded successfully:\n";
    std::cout << "  Total Namespaces:   " << namespaces.size() << "\n";
    std::cout << "  Total Variables:    " << total_vars << "\n";
    std::cout << "\nAll operations completed successfully!\n";
}

int main(int argc, char** argv) {
    try {
        print_header();

        VTCRuntime runtime;

        const std::string file_path = (argc > 1) ? argv[1] : "./samples/comprehensive.vtc";
        std::cout << "\nLoading: " << file_path << "\n";

        runtime.load_file(file_path);
        std::cout << "File loaded successfully!\n";

        print_namespaces(runtime);

        // Demonstrate each namespace category
        demonstrate_system_config(runtime);
        demonstrate_user_data(runtime);
        demonstrate_security(runtime);
        demonstrate_performance(runtime);
        demonstrate_feature_flags(runtime);
        demonstrate_localization(runtime);
        demonstrate_calculations(runtime);
        demonstrate_bitwise_operations(runtime);
        demonstrate_string_operations(runtime);
        demonstrate_advanced_operations(runtime);
        demonstrate_advanced_math(runtime);

        print_summary(runtime);

        std::cout << "\n" << std::string(70, '=') << "\n";
        std::cout << "Comprehensive demo completed successfully!\n\n";

        return 0;

    } catch (const std::exception& e) {
        std::cerr << "\nFatal error: " << e.what() << "\n\n";
        return 1;
    } catch (...) {
        std::cerr << "\nUnknown fatal error occurred\n\n";
        return 1;
    }
}