# Makefile for Linux-compatible RISC-V64 test programs (ch18_file* for tg-ch18 kernel)
# These programs use standard POSIX C APIs to test file operations

CC = gcc
RISC64_CC = riscv64-linux-gnu-gcc
CFLAGS = -static -Wall -Wextra -g -std=c99

# Detection: if cross-compiler is available, use it
ifneq ($(shell which $(RISC64_CC) 2>/dev/null),)
    USE_RISC64 = 1
    FINAL_CC = $(RISC64_CC)
    FINAL_CFLAGS = $(CFLAGS)
    TARGET_ARCH = RISC-V64 (cross)
else
    FINAL_CC = $(CC)
    FINAL_CFLAGS = $(CFLAGS)
    TARGET_ARCH = native
endif

# Programs for tg-ch18 kernel
PROGRAMS_CH18 = ch18_file0 ch18_file1 ch18_file2 ch18_file3
PROGRAMS = $(PROGRAMS_CH18)

SOURCES_CH18 = $(addsuffix .c, $(PROGRAMS_CH18))

OBJECTS_CH18 = $(addsuffix .o, $(PROGRAMS_CH18))
OBJECTS = $(OBJECTS_CH18)

.PHONY: all clean help build-ch18-only

all: $(PROGRAMS_CH18)
	@echo "✅ Build complete for $(TARGET_ARCH)"
	@echo ""
	@echo "Compiled ch18_file* programs:"
	@for prog in $(PROGRAMS_CH18); do \
		if [ -f $$prog ]; then \
			echo "  - $$prog"; \
		fi \
	done
	@echo ""
	@echo "Next steps:"
	@echo "  1. Pack to fs.img: ./pack-to-fsimg.sh"
	@echo "  2. Run with qemu-system-riscv64: ./test-qemu.sh"

build-ch18-only: all


# ch18_file* programs (for tg-ch18 kernel)
ch18_file0: ch18_file0.c
	$(FINAL_CC) $(FINAL_CFLAGS) -o $@ $<

ch18_file1: ch18_file1.c
	$(FINAL_CC) $(FINAL_CFLAGS) -o $@ $<

ch18_file2: ch18_file2.c
	$(FINAL_CC) $(FINAL_CFLAGS) -o $@ $<

ch18_file3: ch18_file3.c
	$(FINAL_CC) $(FINAL_CFLAGS) -o $@ $<

clean:
	rm -f $(OBJECTS) $(PROGRAMS)
	@echo "✅ Cleaned"

help:
	@echo "Linux-compatible RISC-V64 test programs for tg-ch18 kernel"
	@echo ""
	@echo "Targets:"
	@echo "  make (default)     - Build ch18_file* programs"
	@echo "  make clean         - Remove built files"
	@echo "  make help          - Show this help"
	@echo ""
	@echo "Requirements:"
	@echo "  - riscv64-linux-gnu-gcc (for cross-compilation to RISC-V64)"
	@echo "  - qemu-system-riscv64 (for testing with tg-ch18 kernel)"
	@echo ""
	@echo "Quick start:"
	@echo "  make clean && make              # Compile"
	@echo "  ./pack-to-fsimg.sh              # Pack to filesystem"
	@echo "  ./test-qemu.sh                  # Run in QEMU"
	@echo ""
	@echo "Or use the all-in-one script:"
	@echo "  ./quickstart.sh
