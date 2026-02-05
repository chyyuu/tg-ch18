#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>

int main() {
    const char *test_str = "Hello, world!";
    const char *fname = "fname";
    
    // Test 1: Create and write file
    int fd = open(fname, O_CREAT | O_WRONLY, 438);  // 0o666 = 438
    if (fd < 0) {
        perror("open for write failed");
        return 1;
    }
    
    ssize_t written = write(fd, test_str, strlen(test_str));
    if (written < 0) {
        perror("write failed");
        return 1;
    }
    close(fd);
    
    // Test 2: Read the file back
    fd = open(fname, O_RDONLY, 0);
    if (fd < 0) {
        perror("open for read failed");
        return 1;
    }
    
    char buffer[100] = {0};
    ssize_t read_bytes = read(fd, buffer, sizeof(buffer) - 1);
    if (read_bytes < 0) {
        perror("read failed");
        return 1;
    }
    close(fd);
    
    // Verify content
    if (strcmp(test_str, buffer) != 0) {
        fprintf(stderr, "Content mismatch: expected '%s', got '%s'\n", test_str, buffer);
        return 1;
    }
    
    printf("Test file0 OK!\n");
    return 0;
}
