# ZygoteAI — Assembly cell build toolchain
# Requires NASM (https://nasm.us)

NASM = nasm
NASMFLAGS = -f bin -i.

CELLS = sensor logger memory

all: $(CELLS)

%: %.asm common.inc fifo.inc
	$(NASM) $(NASMFLAGS) $< -o $@.bin

clean:
	rm -f *.bin

.PHONY: all clean
