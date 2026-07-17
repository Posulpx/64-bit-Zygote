; ZygoteAI — loopback cell (closes the signaling loop)
; Reads a signal of any type, always emits type_id=1
; with an incremented state counter.
;
; This acts as the "sensor" in the emergence test —
; it resets the type back to 1 so the loop can continue.
;
; Entry: rcx=state, rdx=input_fifo, r8=output_fifo, r9=control

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
    sub rsp, 32
    sub rsp, 64

    mov rbx, rcx          ; state
    mov rsi, rdx          ; input FIFO
    mov rdi, r8           ; output FIFO

    lea r10, [rsp + 32]
    fifo_pop rsi, r9, r10
    jz .done

    ; Increment state counter at state[0..3]
    mov eax, [rbx]
    inc eax
    mov [rbx], eax

    ; Write counter to payload[0..3]
    mov [r10 + SIG_PAYLOAD], eax

    ; Reset type_id to 1 (closes the loop)
    mov byte [r10 + SIG_TYPE_ID], 1

    ; Broadcast
    mov dword [r10 + SIG_TARGET_ID], 0xFFFFFFFF

    ; Push to output FIFO
    mov r11, r9
    fifo_push rdi, r11, r10

.done:
    add rsp, 64
    add rsp, 32
    pop rdi
    pop rsi
    pop rbx
    ret
