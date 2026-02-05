#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>

int main() {
    const char *fname = "fname3";
    const char *test_str = "some random long long long long long long long long string";
    
    for (int i = 0; i < 10; i++) {
        // Open and create file
        int fd = open(fname, O_CREAT | O_WRONLY, 438);  // 0o666 = 438
        if (fd < 0) {
            fprintf(stderr, "Iteration %d: failed to create file\n", i);
            return 1;
        }
        
        // Write data 50 times
        for (int j = 0; j < 50; j++) {
            if (write(fd, test_str, 58) < 0) {
                fprintf(stderr, "Iteration %d: write failed\n", i);
                return 1;
            }
        }
        close(fd);
        
        // Unlink the file
        if (unlink(fname) < 0) {
            fprintf(stderr, "Iteration %d: unlink failed\n", i);
            return 1;
        }
        
        // Verify the file is gone
        fd = open(fname, O_RDONLY, 0);
        if (fd >= 0) {
            fprintf(stderr, "Iteration %d: file should not exist after unlink\n", i);
            close(fd);
            return 1;
        }
        
        printf("test iteration %d\n", i);
    }
    
    printf("Test mass open/unlink OK!\n");
    return 0;
}
