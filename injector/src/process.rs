use crate::utils;
use std::cell::{Ref, RefCell};
use std::error::Error;
use std::ffi::c_void;
use std::ops::Deref;
use std::rc::Rc;

#[cfg(windows)]
extern crate windows;

#[cfg(windows)]
pub fn processes() -> Processes {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS,
    };

    let handle;
    unsafe {
        handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    }

    Processes {
        first: true,
        snapshot_handle: handle.unwrap(),
    }
}

#[cfg(windows)]
pub struct Processes {
    first: bool,
    snapshot_handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Iterator for Processes {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            Process32FirstW, Process32NextW, PROCESSENTRY32W,
        };

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if self.first {
            self.first = false;
            let ret;
            unsafe {
                ret = Process32FirstW(
                    self.snapshot_handle,
                    &mut process_entry as *mut PROCESSENTRY32W,
                );
            }
            if !ret.as_bool() {
                None
            } else {
                Some(Process {
                    id: process_entry.th32ProcessID,
                    name: utils::get_process_name(&process_entry.szExeFile),
                    process_handle: RefCell::new(None),
                    memory_manager: RefCell::new(None),
                })
            }
        } else {
            let ret;
            unsafe {
                ret = Process32NextW(
                    self.snapshot_handle,
                    &mut process_entry as *mut PROCESSENTRY32W,
                );
            }
            if !ret.as_bool() {
                None
            } else {
                Some(Process {
                    id: process_entry.th32ProcessID,
                    name: utils::get_process_name(&process_entry.szExeFile),
                    process_handle: RefCell::new(None),
                    memory_manager: RefCell::new(None),
                })
            }
        }
    }
}

pub struct Process {
    id: u32,
    name: String,
    process_handle: RefCell<Option<Rc<HandleWrapper>>>,
    memory_manager: RefCell<Option<MemoryManager>>,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[cfg(windows)]
    pub fn modules(&self) -> Modules {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPMODULE,
        };

        let handle;
        unsafe {
            handle = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, self.id());
        }

        Modules {
            first: true,
            snapshot_handle: handle.unwrap(),
        }
    }

    /// Returns a handle to the process with `PROCESS_ALL_ACCESS` rights.
    ///
    /// This method uses a lazy pattern so if you never call the `get_process_handle()` function,
    /// `Process` will not have an open process handle.
    #[cfg(windows)]
    fn get_process_handle(&self) -> Result<Rc<HandleWrapper>, Box<dyn Error>> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
        let handle;
        unsafe {
            handle = OpenProcess(PROCESS_ALL_ACCESS, false, self.id)?;
        }

        if self.process_handle.borrow().is_none() {
            *self.process_handle.borrow_mut() = Some(Rc::new(HandleWrapper { handle }));
        }

        Ok(Rc::clone(&Ref::map(
            self.process_handle.borrow(),
            |process_handle| process_handle.as_ref().unwrap(),
        )))
    }

    /// Returns a memory manager to the process, used to read and write into the process memory.
    ///
    /// Since `MemoryManager` needs a handle to the process, the function will call `get_process_handle()`,
    /// opening a process handle to the process.
    #[cfg(windows)]
    pub fn get_memory_manager(
        &self,
    ) -> Result<impl Deref<Target = MemoryManager> + '_, Box<dyn Error>> {
        if (*self.memory_manager.borrow()).is_none() {
            let memory_manager = MemoryManager::new(self.get_process_handle()?);
            *self.memory_manager.borrow_mut() = Some(memory_manager);
        }

        Ok(Ref::map(self.memory_manager.borrow(), |memory_manager| {
            memory_manager.as_ref().unwrap()
        }))
    }

    #[cfg(windows)]
    pub fn get_exports_for_module(
        &self,
        module: &Module,
    ) -> Result<Vec<ExportedFunction>, Box<dyn std::error::Error>> {
        use windows::Win32::System::Diagnostics::Debug::IMAGE_NT_HEADERS64;
        use windows::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY};

        let memory_manager = self.get_memory_manager()?;

        let mut dos_header = IMAGE_DOS_HEADER::default();
        memory_manager.read_from_address(module.load_address, &mut dos_header);

        let mut nt_headers = IMAGE_NT_HEADERS64::default();
        let nt_headers_address = module.load_address + dos_header.e_lfanew as usize;

        memory_manager.read_from_address(nt_headers_address, &mut nt_headers);

        let exports_table_address = module.load_address
            + nt_headers.OptionalHeader.DataDirectory[0].VirtualAddress as usize;

        let mut exports = IMAGE_EXPORT_DIRECTORY::default();
        memory_manager.read_from_address(exports_table_address, &mut exports);

        let base = exports.Base as u16;
        let functions_address = module.load_address + exports.AddressOfFunctions as usize;
        let names_address = module.load_address + exports.AddressOfNames as usize;
        let names_ordinals_map_address =
            module.load_address + exports.AddressOfNameOrdinals as usize;

        let exported_functions = (0..exports.NumberOfNames as usize)
            .into_iter()
            .map(|i| {
                let function_name_addr = names_address + i * 4;
                let function_name_ordinal_map_address = names_ordinals_map_address + i * 2;
                let mut function_name_rva: u32 = 0;
                memory_manager.read_from_address(function_name_addr, &mut function_name_rva);

                let function_name = memory_manager
                    .read_function_name(module.load_address + function_name_rva as usize);

                let mut function_ordinal = 0_u16;
                memory_manager
                    .read_from_address(function_name_ordinal_map_address, &mut function_ordinal);

                let mut function_address_rva = 0_u32;
                memory_manager.read_from_address(
                    functions_address + function_ordinal as usize * 4,
                    &mut function_address_rva,
                );
                let function_address = module.load_address + function_address_rva as usize;

                ExportedFunction {
                    ordinal: (function_ordinal + base) as u32,
                    name: function_name,
                    address: function_address,
                }
            })
            .collect::<Vec<ExportedFunction>>();

        Ok(exported_functions)
    }
}

#[cfg(windows)]
pub struct Modules {
    first: bool,
    snapshot_handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Iterator for Modules {
    type Item = Module;

    fn next(&mut self) -> Option<Self::Item> {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            Module32FirstW, Module32NextW, MODULEENTRY32W,
        };

        let mut module_entry = MODULEENTRY32W {
            dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        if self.first {
            self.first = false;
            let ret;
            unsafe {
                ret = Module32FirstW(
                    self.snapshot_handle,
                    &mut module_entry as *mut MODULEENTRY32W,
                );
            }
            if !ret.as_bool() {
                None
            } else {
                Some(Module {
                    name: utils::get_module_name(&module_entry.szModule),
                    load_address: module_entry.modBaseAddr as usize,
                    size: module_entry.modBaseSize as usize,
                })
            }
        } else {
            let ret;
            unsafe {
                ret = Module32NextW(
                    self.snapshot_handle,
                    &mut module_entry as *mut MODULEENTRY32W,
                );
            }
            if !ret.as_bool() {
                None
            } else {
                Some(Module {
                    name: utils::get_module_name(&module_entry.szModule),
                    load_address: module_entry.modBaseAddr as usize,
                    size: module_entry.modBaseSize as usize,
                })
            }
        }
    }
}

pub struct Module {
    name: String,
    load_address: usize,
    size: usize,
}

impl Module {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn load_address(&self) -> usize {
        self.load_address
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(windows)]
pub struct MemoryManager {
    process_handle: Rc<HandleWrapper>,
    /// Addresses and sizes of the allocations
    allocations: RefCell<
        Vec<(
            usize, /* allocation_address */
            usize, /* allocation_size */
        )>,
    >,
}

#[cfg(windows)]
impl MemoryManager {
    fn new(process_handle: Rc<HandleWrapper>) -> MemoryManager {
        MemoryManager {
            process_handle,
            allocations: RefCell::new(vec![]),
        }
    }

    pub fn read_from_address<T>(&self, address: usize, buffer: &mut T) {
        use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

        let mut bytes_read = 0_usize;
        assert!(!self.process_handle.handle.is_invalid());

        unsafe {
            let ret = ReadProcessMemory(
                self.process_handle.handle,
                address as *const c_void,
                buffer as *mut T as *mut c_void,
                std::mem::size_of::<T>(),
                &mut bytes_read as *mut usize,
            );

            if !ret.as_bool() {
                panic!("error!");
            }
        }
    }

    pub fn allocate_and_write(&self, data: &[u8]) -> Result<usize, Box<dyn Error>> {
        use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
        use windows::Win32::System::Memory::VirtualAllocEx;
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};

        assert!(!self.process_handle.handle.is_invalid());

        let allocated_address;
        unsafe {
            allocated_address = VirtualAllocEx(
                self.process_handle.handle,
                std::ptr::null_mut(),
                data.len(),
                MEM_COMMIT,
                PAGE_EXECUTE_READWRITE,
            );
        }

        if allocated_address.is_null() {
            return Err(Box::new(std::io::Error::last_os_error()));
        }
        self.allocations
            .borrow_mut()
            .push((allocated_address as usize, data.len()));

        let mut bytes_written = 0_usize;
        let ret;
        unsafe {
            ret = WriteProcessMemory(
                self.process_handle.handle,
                allocated_address,
                data.as_ptr() as *const c_void,
                data.len(),
                &mut bytes_written as *mut usize,
            );
        }

        if !ret.as_bool() {
            return Err(Box::new(std::io::Error::last_os_error()));
        }

        println!(
            "{} bytes successfully written to the address {:#x}",
            bytes_written, allocated_address as usize
        );

        Ok(allocated_address as usize)
    }

    fn read_function_name(&self, function_name_address: usize) -> String {
        let mut function_name: Vec<u8> = vec![];
        let mut read_terminator = false;
        let mut offset = 0_usize;
        let mut buffer = [0_u8; std::mem::size_of::<usize>()];

        loop {
            self.read_from_address(function_name_address + offset, &mut buffer);

            function_name.extend(buffer.iter().take_while(|&&c| {
                if c == 0 {
                    read_terminator = true;
                }
                c != 0
            }));

            if read_terminator {
                break;
            }

            offset += std::mem::size_of::<usize>();
        }

        String::from_utf8(function_name).unwrap()
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        use windows::Win32::System::Memory::VirtualFreeEx;
        use windows::Win32::System::Memory::MEM_DECOMMIT;

        for allocation in &*self.allocations.borrow() {
            assert!(!self.process_handle.handle.is_invalid());
            let allocation_address = (allocation.0) as *mut c_void;
            let allocation_size = allocation.1;
            let ret;
            unsafe {
                ret = VirtualFreeEx(
                    self.process_handle.handle,
                    allocation_address,
                    allocation_size,
                    MEM_DECOMMIT,
                );
            }
            if !ret.as_bool() {
                panic!("Uh oh something went wrong and we are leaking");
            }
            println!("Dropped {} allocated bytes", allocation_size);
        }
    }
}

#[derive(Debug)]
pub struct ExportedFunction {
    ordinal: u32,
    name: String,
    address: usize,
}

impl ExportedFunction {
    pub fn ordinal(&self) -> u32 {
        self.ordinal
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn address(&self) -> usize {
        self.address
    }
}

impl std::fmt::Display for ExportedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({:#02X})", self.ordinal, self.name, self.address)
    }
}

#[cfg(windows)]
struct HandleWrapper {
    handle: windows::Win32::Foundation::HANDLE,
}

impl From<windows::Win32::Foundation::HANDLE> for HandleWrapper {
    fn from(handle: windows::Win32::Foundation::HANDLE) -> Self {
        HandleWrapper { handle }
    }
}

impl From<HandleWrapper> for windows::Win32::Foundation::HANDLE {
    fn from(wrapper: HandleWrapper) -> Self {
        wrapper.handle
    }
}

impl Deref for HandleWrapper {
    type Target = windows::Win32::Foundation::HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        println!("Dropping handle");
        use windows::Win32::Foundation::CloseHandle;
        let ret;
        unsafe {
            ret = CloseHandle(self.handle);
        }
        if !ret.as_bool() {
            panic!("Error during process handle close.");
        }
    }
}
