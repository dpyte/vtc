#ifndef VTC_RUNTIME_H
#define VTC_RUNTIME_H

#include <stdint.h>

typedef struct Runtime Runtime;

Runtime *vtc_runtime_new();
Runtime *vtc_runtime_from(const char *path);
int vtc_runtime_load_file(Runtime *runtime, const char *path);
void vtc_runtime_destroy(Runtime *runtime);

#endif // VTC_RUNTIME_H