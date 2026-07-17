; ZygoteAI — relay cell
; Reads a signal, increments the type_id by 1, increments a state
; counter, writes the counter to payload[0..3], and broadcasts it.
; This creates a chain: type N → type N+1.
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

    lea r10, [rsp + 32]   ; local signal buffer
    fifo_pop rsi, r9, r10
    jz .done

    ; Increment state counter at state[0..3]
    mov eax, [rbx]
    inc eax
    mov [rbx], eax

    ; Write counter to payload[0..3]
    mov [r10 + SIG_PAYLOAD], eax

    ; Increment type_id by 1
    mov al, [r10 + SIG_TYPE_ID]
    inc al
    mov [r10 + SIG_TYPE_ID], al

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
