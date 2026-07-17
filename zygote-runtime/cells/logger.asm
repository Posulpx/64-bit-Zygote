; ZygoteAI — logger cell
; Reads signals from input FIFO and writes their type_id + first 8 payload
; bytes into a fixed address in the state buffer. Sets state[255] = 0x01
; to signal "data ready" to the Rust runtime.
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
    sub rsp, 32        ; shadow space
    sub rsp, 64        ; local signal buffer

    mov rbx, rcx       ; state
    mov rsi, rdx       ; input FIFO

    lea r10, [rsp + 32]
    fifo_pop rsi, r9, r10
    jz .done

    ; Write type_id to state[0]
    mov al, [r10 + SIG_TYPE_ID]
    mov [rbx], al

    ; Write first 8 payload bytes to state[8..16]
    mov rax, [r10 + SIG_PAYLOAD]
    mov [rbx + 8], rax

    ; Write source_id to state[16..20]
    mov eax, [r10 + SIG_SOURCE_ID]
    mov [rbx + 16], eax

    ; Set ready flag at state[255]
    mov byte [rbx + 255], 1

.done:
    add rsp, 64
    add rsp, 32
    pop rsi
    pop rbx
    ret
