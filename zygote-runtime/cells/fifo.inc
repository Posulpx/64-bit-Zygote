; ZygoteAI — Ring buffer FIFO helpers for Assembly cells
; Include after common.inc
;
; Macros:
;   fifo_pop  fifo_base, ctrl_base, [dst]   — pop a signal from a FIFO
;   fifo_push fifo_base, ctrl_base, [src]   — push a signal to a FIFO
;
; Registers used: rax, rcx
; All other registers preserved (including r10, r11).

; Pop one signal from FIFO at fifo_base, controlled by ctrl_base.
; On success: reads signal into memory at dst (must be 64B aligned), ZF=0
; On empty:   dst unchanged, ZF=1
; Trashes: rax, rcx
%macro fifo_pop 3
    ; %1 = fifo_base  (rdx for input, r8 for output)
    ; %2 = ctrl_base  (r9)
    ; %3 = dst        (memory, 64 bytes)

    ; Load head and tail from control block
    movzx eax, word [%2 + CTRL_INPUT_HEAD]
    movzx ecx, word [%2 + CTRL_INPUT_TAIL]

    cmp ax, cx
    je %%empty

    ; Compute offset = (head & FIFO_MASK) * SIGNAL_SIZE
    and eax, FIFO_MASK
    imul eax, SIGNAL_SIZE

    ; Copy signal from FIFO[offset] to dst
    mov rcx, [%1 + rax]
    mov [%3], rcx
    mov rcx, [%1 + rax + 8]
    mov [%3 + 8], rcx
    mov rcx, [%1 + rax + 16]
    mov [%3 + 16], rcx
    mov rcx, [%1 + rax + 24]
    mov [%3 + 24], rcx
    mov rcx, [%1 + rax + 32]
    mov [%3 + 32], rcx
    mov rcx, [%1 + rax + 40]
    mov [%3 + 40], rcx
    mov rcx, [%1 + rax + 48]
    mov [%3 + 48], rcx
    mov rcx, [%1 + rax + 56]
    mov [%3 + 56], rcx

    ; head = (head + 1) & FIFO_MASK
    movzx eax, word [%2 + CTRL_INPUT_HEAD]
    inc eax
    and eax, FIFO_MASK
    mov [%2 + CTRL_INPUT_HEAD], ax

    xor eax, eax
    inc eax          ; set ZF=0 to indicate success
    jmp %%done

%%empty:
    xor eax, eax     ; set ZF=1
%%done:
%endmacro

; Push one signal from src to FIFO at fifo_base, controlled by ctrl_base.
; On success: ZF=0
; On full:    nothing written, ZF=1
; Trashes: rax, rcx
%macro fifo_push 3
    ; %1 = fifo_base
    ; %2 = ctrl_base
    ; %3 = src (memory, 64 bytes)

    movzx eax, word [%2 + CTRL_OUTPUT_HEAD]
    movzx ecx, word [%2 + CTRL_OUTPUT_TAIL]

    ; Check if full: ((tail + 1) & FIFO_MASK) == head
    lea eax, [ecx + 1]
    and eax, FIFO_MASK
    cmp ax, word [%2 + CTRL_OUTPUT_HEAD]
    je %%full

    ; Compute offset = (tail & FIFO_MASK) * SIGNAL_SIZE
    mov eax, ecx
    and eax, FIFO_MASK
    imul eax, SIGNAL_SIZE

    ; Copy signal from src to FIFO[offset]
    mov rcx, [%3]
    mov [%1 + rax], rcx
    mov rcx, [%3 + 8]
    mov [%1 + rax + 8], rcx
    mov rcx, [%3 + 16]
    mov [%1 + rax + 16], rcx
    mov rcx, [%3 + 24]
    mov [%1 + rax + 24], rcx
    mov rcx, [%3 + 32]
    mov [%1 + rax + 32], rcx
    mov rcx, [%3 + 40]
    mov [%1 + rax + 40], rcx
    mov rcx, [%3 + 48]
    mov [%1 + rax + 48], rcx
    mov rcx, [%3 + 56]
    mov [%1 + rax + 56], rcx

    ; tail = (tail + 1) & FIFO_MASK
    movzx eax, word [%2 + CTRL_OUTPUT_TAIL]
    inc eax
    and eax, FIFO_MASK
    mov [%2 + CTRL_OUTPUT_TAIL], ax

    xor eax, eax
    inc eax          ; ZF=0
    jmp %%done

%%full:
    xor eax, eax     ; ZF=1
%%done:
%endmacro
