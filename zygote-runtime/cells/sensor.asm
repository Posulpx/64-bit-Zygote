; ZygoteAI — sensor cell
; Reads signals from input FIFO, increments a counter in state,
; forwards the signal to output FIFO with an updated payload.
;
; Entry: rcx=state, rdx=input_fifo, r8=output_fifo, r9=control
; State layout:
;   [0..3]   counter (u32, little-endian)

bits 64
default rel

%include "common.inc"
%include "fifo.inc"

section .text
global cell_entry
cell_entry:
    push rbx
    push rsi
    push rdi
    sub rsp, 32        ; shadow space for potential calls
    sub rsp, 64        ; local signal buffer (aligned)

    mov rbx, rcx       ; state
    mov rsi, rdx       ; input FIFO
    mov rdi, r8        ; output FIFO
    ; r9 stays as control

    ; Try to pop a signal from input FIFO
    lea r10, [rsp + 32]  ; local buffer for the signal
    fifo_pop rsi, r9, r10
    jz .done

    ; Increment counter in state
    mov eax, [rbx]
    inc eax
    mov [rbx], eax

    ; Write counter into signal payload bytes 0..3
    mov [r10 + SIG_PAYLOAD], eax

    ; Set target to broadcast so it routes by affinity to all listeners
    mov dword [r10 + SIG_TARGET_ID], 0xFFFFFFFF

    ; Forward signal to output FIFO
    mov r11, r9          ; control block
    fifo_push rdi, r11, r10

.done:
    add rsp, 64
    add rsp, 32
    pop rdi
    pop rsi
    pop rbx
    ret
