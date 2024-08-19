# lc3-vm
Implementation of lc3-vm in Rust

## Execution
- `make run FILE=file_name` to run an object file, `file_name` contains the program to execute, else use `make run` to use the example `2048.obj` file.
- `make interactive` to open an interactive console of the Virtual Machine, where all the instructions you type in binary form will be executed at the moment, also if you type "r", you can see the state of the registers.
- `make debug FILE=file_name` to run an object file in debug mode, `file_name` contains the program to debug. If you just use `make debug`, it will use the `2048.obj` by default. In another console you have to type `make debugger` to use the debugger there

### Debugger Controls
- "n" to pass one instruction and see the instruction executed on the debugger
- "Any u16 number" to execute that amount of instructions (for example, just type 10000 to pass 10000 instructions forward, recommended for the `2048.obj`)
- "r" to see the state of the registers
