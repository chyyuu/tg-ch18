#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <stdio.h>
#include <assert.h>

int main() {
    const char *fname = "fname2";
    const char *lname0 = "linkname0";
    const char *lname1 = "linkname1";
    const char *lname2 = "linkname2";
    struct stat st, st2;
    const char *test_str = "Hello, world!";
    
    // Create original file
    int fd = open(fname, O_CREAT | O_WRONLY, 438);  // 0o666 = 438
    if (fd < 0) {
        perror("open failed");
        return 1;
    }
    
    // Create first link
    if (link(fname, lname0) < 0) {
        perror("link failed");
        return 1;
    }
    
    // Check nlink
    if (fstat(fd, &st) < 0) {
        perror("fstat failed");
        return 1;
    }
    
    if (st.st_nlink != 2) {
        fprintf(stderr, "Expected nlink=2, got %ld\n", st.st_nlink);
        return 1;
    }
    
    // Create more links
    if (link(fname, lname1) < 0 || link(fname, lname2) < 0) {
        perror("link failed");
        return 1;
    }
    
    // Check nlink again
    if (fstat(fd, &st) < 0) {
        perror("fstat failed");
        return 1;
    }
    
    if (st.st_nlink != 4) {
        fprintf(stderr, "Expected nlink=4 after creating more links, got %ld\n", st.st_nlink);
        return 1;
    }
    
    // Write data
    write(fd, test_str, 13);
    close(fd);
    
    // Unlink original file
    if (unlink(fname) < 0) {
        perror("unlink failed");
        return 1;
    }
    
    // Open first link for reading
    fd = open(lname0, O_RDONLY, 0);
    if (fd < 0) {
        perror("open link for read failed");
        return 1;
    }
    
    char buffer[100] = {0};
    ssize_t read_bytes = read(fd, buffer, sizeof(buffer) - 1);
    if (read_bytes < 0) {
        perror("read failed");
        return 1;
    }
    
    // Check stats
    if (fstat(fd, &st2) < 0) {
        perror("fstat failed");
        return 1;
    }
    
    if (st2.st_dev != st.st_dev || st2.st_ino != st.st_ino) {
        fprintf(stderr, "Inode mismatch\n");
        return 1;
    }
    
    if (st2.st_nlink != 3) {
        fprintf(stderr, "Expected nlink=3 after unlinking original, got %ld\n", st2.st_nlink);
        return 1;
    }
    
    // Cleanup
    close(fd);
    unlink(lname1);
    unlink(lname2);
    
    if (fstat(fd, &st2) < 0) {
        // File might be closed, reopen and check
        fd = open(lname0, O_RDONLY, 0);
        if (fd >= 0) {
            fstat(fd, &st2);
            if (st2.st_nlink != 1) {
                fprintf(stderr, "Expected nlink=1 after unlinking all but one, got %ld\n", st2.st_nlink);
            }
            close(fd);
        }
    }
    
    unlink(lname0);
    printf("Test link OK!\n");
    return 0;
}
