#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <stdio.h>
#include <assert.h>

int main() {
    const char *fname = "fname";
    struct stat st;
    
    // Create a file
    int fd = open(fname, O_CREAT | O_WRONLY, 420);  // 0o644 = 420
    if (fd < 0) {
        perror("open failed");
        return 1;
    }
    
    // Write some data
    write(fd, "Hello", 5);
    
    // Test fstat
    if (fstat(fd, &st) < 0) {
        perror("fstat failed");
        return 1;
    }
    
    printf("File info:\n");
    printf("  st_size: %ld\n", st.st_size);
    printf("  st_mode: 0o%o\n", st.st_mode & 0777);
    printf("  st_nlink: %ld\n", st.st_nlink);
    printf("  st_ino: %ld\n", st.st_ino);
    
    // Verify the file size
    if (st.st_size != 5) {
        fprintf(stderr, "File size mismatch: expected 5, got %ld\n", st.st_size);
        close(fd);
        return 1;
    }
    
    // Verify it's a regular file
    if (!S_ISREG(st.st_mode)) {
        fprintf(stderr, "Not a regular file\n");
        close(fd);
        return 1;
    }
    
    close(fd);
    printf("Test file1 OK!\n");
    return 0;
}
