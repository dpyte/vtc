#include <iostream>
#include "vtc.hxx"
#include <cassert>
#include <cstring>

using namespace VTC;

int main(int argc, const char **argv) {
    CRuntime runtime = runtime_new();

    const char* vtc_content =
        "@test_namespace:\n"
        "    $test_int := 42\n"
        "    $test_float := 3.14\n"
        "    $test_string := \"Hello, World!\"\n"
        "    $test_list := [1, 2, 3]\n";

    int result = runtime_load_vtc(runtime, vtc_content);
    assert(result == 0);

    int64_t int_value;
    result = runtime_get_integer(runtime, "test_namespace", "test_int", &int_value);
    assert(result == 0);
    assert(int_value == 42);

    double float_value;
    result = runtime_get_float(runtime, "test_namespace", "test_float", &float_value);
    assert(result == 0);
    assert(float_value == 3.14);

    char* string_value = runtime_get_string(runtime, "test_namespace", "test_string");
    assert(string_value != NULL);
    assert(strcmp(string_value, "Hello, World!") == 0);
    runtime_free_string(string_value);

    VTC::Value* list_value;
    size_t list_length;
    result = runtime_get_list(runtime, "test_namespace", "test_list", &list_value, &list_length);
    assert(result == 0);
    assert(list_length == 3);
    runtime_free_value_array(list_value, list_length);

    runtime_free(runtime);

    std::cout << int_value << '\n';
    printf("All tests passed!\n");
    return 0;
}