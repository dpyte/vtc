#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include "vtc_runtime.h"

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

    vtc_runtime_destroy(runtime);
    return 0;
}