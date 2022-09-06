use std::error::Error;
use std::ffi::c_void;

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
                    name: get_process_name(&process_entry.szExeFile),
                    memory_manager: None,
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
                    name: get_process_name(&process_entry.szExeFile),
                    memory_manager: None,
                })
            }
        }
    }
}

fn get_process_name(process_name_raw: &[u16]) -> String {
    let len = process_name_raw.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&process_name_raw[..len]).unwrap()
}

#[cfg(windows)]
pub struct MemoryManager {
    process_handle: windows::Win32::Foundation::HANDLE,
    /// Addresses and sizes of the allocations
    allocations: Vec<(
        usize, /* allocation_address */
        usize, /* allocation_size */
    )>,
}

#[cfg(windows)]
impl MemoryManager {
    pub fn read_from_address<T>(&self, address: usize, buffer: &mut T) {
        use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

        let mut bytes_read = 0_usize;
        unsafe {
            let ret = ReadProcessMemory(
                self.process_handle,
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

    pub fn allocate_and_write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
        use windows::Win32::System::Memory::VirtualAllocEx;
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};

        let allocated_address;
        unsafe {
            allocated_address = VirtualAllocEx(
                self.process_handle,
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
            .push((allocated_address as usize, data.len()));

        let mut bytes_written = 0_usize;
        let ret;
        unsafe {
            ret = WriteProcessMemory(
                self.process_handle,
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

        Ok(())
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        use windows::Win32::System::Memory::VirtualFreeEx;
        use windows::Win32::System::Memory::MEM_DECOMMIT;
        use windows::Win32::Foundation::CloseHandle;

        for allocation in &self.allocations {
            let allocation_address = (allocation.0) as *mut c_void;
            let allocation_size = allocation.1;
            let ret;
            unsafe {
                ret = VirtualFreeEx(
                    self.process_handle,
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

        let ret;
        unsafe {
            ret = CloseHandle(self.process_handle);
        }

        if !ret.as_bool() {
            panic!("Couldnt close the process handle");
        }
    }
}

pub struct Process {
    id: u32,
    name: String,
    memory_manager: Option<MemoryManager>,
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

    #[cfg(windows)]
    fn get_memory_manager(&mut self) -> Result<&mut MemoryManager, Box<dyn Error>> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
        if self.memory_manager.is_none() {
            let process_handle;
            unsafe {
                process_handle = OpenProcess(PROCESS_ALL_ACCESS, false, self.id);
            }

            match process_handle {
                Ok(handle) => {
                    self.memory_manager = Some(MemoryManager {
                        process_handle: handle,
                        allocations: vec![],
                    });
                    Ok(self.memory_manager.as_mut().unwrap())
                }
                Err(e) => Err(Box::new(e)),
            }
        } else {
            Ok(self.memory_manager.as_mut().unwrap())
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
                    name: get_module_name(&module_entry.szModule),
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
                    name: get_module_name(&module_entry.szModule),
                    load_address: module_entry.modBaseAddr as usize,
                    size: module_entry.modBaseSize as usize,
                })
            }
        }
    }
}

fn get_module_name(raw_module_name: &[u16]) -> String {
    let len = raw_module_name.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&raw_module_name[..len]).unwrap()
}
