; ZygoteAI — memory cell
; Stores the last N signal types in its state buffer and, upon detecting a
; warning signal (type 9) in recent history, emits a predictive protective
; broadcast (type 50) BEFORE mortality rises. This is the "predictive"
; strategy from the L#056 experiment.
;
; State layout (256 bytes, rcx):
;   [0]    write_count  (u8)  — number of signals seen (mod 256)
;   [1]    warned_flag  (u8)  — 1 once we've emitted type 50 for current episode
;   [16..31] history    (16 u8) — ring buffer of last 16 signal type_ids
;
; Entry: rcx=state, rdx=input_fifo, r8=output_fifo, r9=control

bits 64
default rel

%include "common.inc"
%include "fifo.inc"

HISTORY_BASE   equ 16
HISTORY_LEN    equ 16
WARN_TYPE      equ 9
PROTECT_TYPE   equ 50

section .text
global cell_entry
cell_entry:
    push rbx
    push rsi
    push rdi
    push r12
    sub rsp, 32
    sub rsp, 64

    mov rbx, rcx          ; state
    mov rsi, rdx          ; input FIFO
    mov rdi, r8           ; output FIFO

    ; --- Pop a signal (if any) ---
    lea r10, [rsp + 32]   ; local signal buffer
    fifo_pop rsi, r9, r10
    jz .check_predict

    ; Read the signal type_id (1 byte at offset 8)
    mov al, [r10 + SIG_TYPE_ID]

    ; Store into history ring buffer at HISTORY_BASE + (write_count % 16)
    movzx ecx, byte [rbx]          ; write_count
    and ecx, HISTORY_LEN - 1       ; mod 16
    mov [rbx + HISTORY_BASE + rcx], al

    ; Increment write_count
    movzx ecx, byte [rbx]
    inc ecx
    mov [rbx], cl

.check_predict:
    ; Scan history for WARN_TYPE (9)
    xor r12, r12                    ; found flag
    xor ecx, ecx
.scan:
    cmp cl, HISTORY_LEN
    jge .scan_done
    movzx edx, byte [rbx + HISTORY_BASE + rcx]
    cmp dl, WARN_TYPE
    je .found_warn
    inc ecx
    jmp .scan
.found_warn:
    mov r12, 1
.scan_done:

    ; If warning seen and not yet warned this episode -> emit protective signal
    cmp r12, 1
    jne .done
    movzx ecx, byte [rbx + 1]      ; warned_flag
    cmp cl, 1
    je .done

    ; Set warned_flag = 1
    mov byte [rbx + 1], 1

    ; Build protective broadcast signal type 50
    lea r11, [rsp + 32]
    ; zero the signal buffer
    xor eax, eax
    mov rcx, r11
    mov [rcx], rax
    mov [rcx + 8], rax
    mov [rcx + 16], rax
    mov [rcx + 24], rax
    mov [rcx + 32], rax
    mov [rcx + 40], rax
    mov [rcx + 48], rax
    mov [rcx + 56], rax

    mov byte [r11 + SIG_TYPE_ID], PROTECT_TYPE
    mov dword [r11 + SIG_TARGET_ID], 0xFFFFFFFF   ; broadcast

    mov r12, r9
    fifo_push rdi, r12, r11

    jmp .done

.done:
    add rsp, 64
    add rsp, 32
    pop r12
    pop rdi
    pop rsi
    pop rbx
    ret
