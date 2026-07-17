bits 64
default rel

section .text
global cell_entry
cell_entry:
    mov byte [r9 + 63], 1
    ret
