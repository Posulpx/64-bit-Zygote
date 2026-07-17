; ZygoteAI — Common definitions for Assembly cells
; Windows x64 calling convention: rcx=state, rdx=input, r8=output, r9=control
; NASM syntax

; Signal struct offsets (packed, 64 bytes)
SIGNAL_SIZE      equ 64
SIG_SOURCE_ID    equ 0   ; u32
SIG_TARGET_ID    equ 4   ; u32
SIG_TYPE_ID      equ 8   ; u8
SIG_PAYLOAD      equ 9   ; 55 bytes

; Control block offsets (64 bytes)
CTRL_INPUT_HEAD  equ 0   ; u16
CTRL_INPUT_TAIL  equ 2   ; u16
CTRL_OUTPUT_HEAD equ 4   ; u16
CTRL_OUTPUT_TAIL equ 6   ; u16
CTRL_KILL_FLAG   equ 63  ; u8

; FIFO constants
FIFO_CAPACITY    equ 4
FIFO_SIZE        equ SIGNAL_SIZE * FIFO_CAPACITY  ; 256

; Ring buffer mask (for power-of-2 wraparound)
FIFO_MASK        equ FIFO_CAPACITY - 1  ; 3

; Status return values
CELL_OK          equ 0
CELL_KILL        equ 1

; Usage at cell entry:
;   rcx = state base pointer
;   rdx = input FIFO base pointer
;   r8  = output FIFO base pointer
;   r9  = control block base pointer
;
; All registers are caller-saved except rbx, rbp, rdi, rsi, r12-r15.
; The cell must preserve those if it uses them.
