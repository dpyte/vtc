#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include "vtc_runtime.h"

void print_namespaces(Runtime* runtime) {
    char** namespaces = NULL;
    size_t count = 0;
    if (vtc_runtime_list_namespaces(runtime, &namespaces, &count) == 0) {
        printf("Namespaces:\n");
        for (size_t i = 0; i < count; i++) {
            printf("  %s\n", namespaces[i]);
        }
        vtc_runtime_free_string_array(namespaces, count);
    } else {
        fprintf(stderr, "Failed to list namespaces\n");
    }
}

void print_variables(Runtime* runtime, const char* namespace) {
    char** variables;
    size_t count;
    if (vtc_runtime_list_variables(runtime, namespace, &variables, &count) == 0) {
        printf("Variables in namespace '%s':\n", namespace);
        for (size_t i = 0; i < count; i++) {
            printf("  %s\n", variables[i]);
        }
        vtc_runtime_free_string_array(variables, count);
    } else {
        fprintf(stderr, "Failed to list variables in namespace '%s'\n", namespace);
    }
}

int main(int argc, const char **argv) {
    Runtime* runtime = vtc_runtime_new();
    if (runtime == NULL) {
        fprintf(stderr, "Failed to create runtime\n");
        return 1;
    }

    const char* file_path = "./samples/intrinsics.vtc";
    int result = vtc_runtime_load_file(runtime, file_path);
    if (result != 0) {
        fprintf(stderr, "Failed to load file: %s (errno: %d)\n", file_path, errno);
        vtc_runtime_destroy(runtime);
        return 1;
    }

    printf("File loaded successfully: %s\n", file_path);

    print_namespaces(runtime);

    const char* test_namespace = "test";
    print_variables(runtime, test_namespace);

    char* str_value;
    int64_t int_value;
    double float_value;
    bool bool_value;

    if ((str_value = vtc_runtime_get_string(runtime, test_namespace, "string_var")) != NULL) {
        printf("String value: %s\n", str_value);
        vtc_runtime_free_string(str_value);
    }

    if (vtc_runtime_get_integer(runtime, test_namespace, "int_var", &int_value) == 0) {
        printf("Integer value: %lld\n", (long long)int_value);
    }

    if (vtc_runtime_get_float(runtime, test_namespace, "float_var", &float_value) == 0) {
        printf("Float value: %f\n", float_value);
    }

    if (vtc_runtime_get_boolean(runtime, test_namespace, "bool_var", &bool_value) == 0) {
        printf("Boolean value: %s\n", bool_value ? "true" : "false");
    }

    Value* list_values;
    size_t list_length;
    if (vtc_runtime_get_list(runtime, test_namespace, "list_var", &list_values, &list_length) == 0) {
        printf("List values:\n");
        for (size_t i = 0; i < list_length; i++) {
            char* value_str = vtc_value_to_string(&list_values[i]);
            printf("  %s\n", value_str);
            vtc_runtime_free_string(value_str);
        }
        vtc_runtime_free_value_array(list_values, list_length);
    }

    vtc_runtime_destroy(runtime);
    return 0;
}