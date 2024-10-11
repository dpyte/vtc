#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


namespace VTC {

constexpr static const size_t SMALL_VEC_SIZE = 64;

template<typename T = void>
struct Rc;

/// A struct representing the runtime environment of a software program.
struct Runtime;

struct Value;

struct CRuntime {
    Runtime *_0;

    CRuntime(Runtime *const& _0)
      : _0(_0)
    {}

};


extern "C" {

void *runtime_as_dict(CRuntime runtime, const char *namespace_, const char *variable);

int runtime_flatten_list(CRuntime runtime,
                         const char *namespace_,
                         const char *variable,
                         Value **result,
                         size_t *length);

void runtime_free(CRuntime runtime);

void runtime_free_dict(void *dict);

void runtime_free_string(char *s);

void runtime_free_string_array(char **arr, size_t length);

void runtime_free_value_array(Rc<Value> *arr, size_t length);

CRuntime runtime_from(const char *path);

int runtime_get_boolean(CRuntime runtime,
                        const char *namespace_,
                        const char *variable,
                        bool *result);

int runtime_get_float(CRuntime runtime,
                      const char *namespace_,
                      const char *variable,
                      double *result);

int runtime_get_integer(CRuntime runtime,
                        const char *namespace_,
                        const char *variable,
                        int64_t *result);

int runtime_get_list(CRuntime runtime,
                     const char *namespace_,
                     const char *variable,
                     Rc<Value> **result,
                     size_t *length);

char *runtime_get_string(CRuntime runtime, const char *namespace_, const char *variable);

int runtime_list_namespaces(CRuntime runtime, char ***result, size_t *length);

int runtime_list_variables(CRuntime runtime,
                           const char *namespace_,
                           char ***result,
                           size_t *length);

int runtime_load_file(CRuntime runtime, const char *path);

int runtime_load_vtc(CRuntime runtime, const char *input);

CRuntime runtime_new();

}  // extern "C"

}  // namespace VTC
