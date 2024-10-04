#ifndef RUNTIME_FFI_H
#define RUNTIME_FFI_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque struct for Runtime
typedef struct CRuntime CRuntime;

// Opaque struct for Value
typedef struct Value Value;

// Create a new Runtime
CRuntime* runtime_new(void);

// Create a Runtime from a file
CRuntime* runtime_from(const char* path);

// Load a file into the Runtime
int runtime_load_file(CRuntime* runtime, const char* path);

// Load VTC content into the Runtime
int runtime_load_vtc(CRuntime* runtime, const char* input);

// Get a string value from the Runtime
char* runtime_get_string(CRuntime* runtime, const char* namespace, const char* variable);

// Get an integer value from the Runtime
int runtime_get_integer(CRuntime* runtime, const char* namespace, const char* variable, int64_t* result);

// Get a float value from the Runtime
int runtime_get_float(CRuntime* runtime, const char* namespace, const char* variable, double* result);

// Get a boolean value from the Runtime
int runtime_get_boolean(CRuntime* runtime, const char* namespace, const char* variable, bool* result);

// Get a list from the Runtime
int runtime_get_list(CRuntime* runtime, const char* namespace, const char* variable, Value** result, size_t* length);

// Convert a list to a dictionary
void* runtime_as_dict(CRuntime* runtime, const char* namespace, const char* variable);

// Flatten a list in the Runtime
int runtime_flatten_list(CRuntime* runtime, const char* namespace, const char* variable, Value** result, size_t* length);

// List all namespaces in the Runtime
int runtime_list_namespaces(CRuntime* runtime, char*** result, size_t* length);

// List all variables in a namespace
int runtime_list_variables(CRuntime* runtime, const char* namespace, char*** result, size_t* length);

// Free the Runtime
void runtime_free(CRuntime* runtime);

// Free a string returned by the Runtime
void runtime_free_string(char* s);

// Free an array of strings returned by the Runtime
void runtime_free_string_array(char** arr, size_t length);

// Free an array of Values returned by the Runtime
void runtime_free_value_array(Value* arr, size_t length);

// Free a dictionary returned by the Runtime
void runtime_free_dict(void* dict);

#ifdef __cplusplus
}
#endif

#endif // RUNTIME_FFI_H