/// Implementation of the Assembler was made possible by
/// [this amazing Online x86 / x64 Assembler and Disassembler](https://defuse.ca/online-x86-assembler.htm)
///
/// The x64 Application Binary Interface (ABI) uses a four-register fast-call calling convention by default.
/// Space is allocated on the call stack as a shadow store for callees to save those registers.
/// Integer arguments are passed in registers RCX, RDX, R8, and R9.
pub struct Assembler {
    data: Vec<u8>,
}

impl Assembler {
    /// Returns a new `Assembler` with no machine code generated.
    pub fn new() -> Assembler {
        Assembler { data: vec![] }
    }

    /// Substracts a value from the stack pointer register.
    ///
    /// The asssembly code emitted in binary is: ```sub rsp, value```
    pub fn sub_rsp(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x48, 0x83, 0xEC]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Adds a value to the stack pointer register.
    ///
    /// The asssembly code emitted in binary is: ```add rsp, value```
    pub fn add_rsp(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x48, 0x83, 0xC4]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Moves a value to the rax register.
    ///
    /// The asssembly code emitted in binary is: ```mov rax, value```
    pub fn mov_rax(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x48, 0xB8]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Moves a value to the rcx register.
    ///
    /// The asssembly code emitted in binary is: ```mov rcx, value```
    pub fn mov_rcx(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x48, 0xB9]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Moves a value to the rdx register.
    ///
    /// The asssembly code emitted in binary is: ```mov rdx, value```
    pub fn mov_rdx(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x48, 0xBA]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Moves a value to the r8 register.
    ///
    /// The asssembly code emitted in binary is: ```mov r8, value```
    pub fn mov_r8(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x49, 0xB8]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Moves a value to the r9 register.
    ///
    /// The asssembly code emitted in binary is: ```mov r9, value```
    pub fn mov_r9(&mut self, value: usize) {
        self.data.extend_from_slice(&[0x49, 0xB9]);
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// Near calls a function with its address specified in rax register.
    ///
    /// The assembly code emitted in binary is: ```call rax```
    pub fn call_rax(&mut self) {
        self.data.extend_from_slice(&[0xFF, 0xD0]);
    }

    /// Moves the value of the rax register into the address specified by the caller.
    ///
    /// The assemly code emitted in binary is: ```mov [address], rax```
    pub fn mov_rax_to(&mut self, address: usize) {
        self.data.extend_from_slice(&[0x48, 0xA3]);
        self.data.extend_from_slice(&address.to_le_bytes());
    }

    /// Near returns to calling procedure.
    ///
    /// The assembly code emitted in binary is: ```ret```
    pub fn ret(&mut self) {
        self.data.extend_from_slice(&[0xC3]);
    }

    /// Retrieves the generated assembly machine code.
    pub fn data(&self) -> &[u8] {
        &self.data
    }


}
