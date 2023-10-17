use crate::utils;
use crate::Result;

use async_std::stream::Stream;
use async_std::sync::Mutex;
use futures::future::join_all;
use std::ffi::c_void;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[cfg(windows)]
extern crate windows;

#[cfg(windows)]
pub(crate) fn processes() -> Processes {
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
pub(crate) struct Processes {
    first: bool,
    snapshot_handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Stream for Processes {
    type Item = Process;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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
                Poll::Ready(None)
            } else {
                Poll::Ready(Some(Process {
                    id: process_entry.th32ProcessID,
                    name: utils::get_process_name(&process_entry.szExeFile),
                    process_handle: Mutex::new(None),
                    memory_manager: Mutex::new(None),
                }))
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
                Poll::Ready(None)
            } else {
                Poll::Ready(Some(Process {
                    id: process_entry.th32ProcessID,
                    name: utils::get_process_name(&process_entry.szExeFile),
                    process_handle: Mutex::new(None),
                    memory_manager: Mutex::new(None),
                }))
            }
        }
    }
}

pub(crate) struct Process {
    id: u32,
    name: String,
    process_handle: Mutex<Option<Arc<HandleWrapper>>>,
    memory_manager: Mutex<Option<Arc<MemoryManager>>>,
}

impl Process {
    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    #[cfg(windows)]
    pub(crate) fn modules(&self) -> Modules {
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
    async fn get_process_handle(&self) -> Result<Arc<HandleWrapper>> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
        let handle;
        unsafe {
            handle = OpenProcess(PROCESS_ALL_ACCESS, false, self.id)?;
        }

        let mut process_handle = self.process_handle.lock().await;
        if process_handle.is_none() {
            *process_handle = Some(Arc::new(HandleWrapper { handle }));
        }

        Ok(Arc::clone(process_handle.as_ref().unwrap()))
    }

    /// Returns a memory manager to the process, used to read and write into the process memory.
    ///
    /// Since `MemoryManager` needs a handle to the process, the function will call `get_process_handle()`,
    /// opening a process handle to the process.
    #[cfg(windows)]
    pub(crate) async fn get_memory_manager(&self) -> Result<Arc<MemoryManager>> {
        let mut memory_manager = self.memory_manager.lock().await;
        if memory_manager.is_none() {
            let process_handle = self.get_process_handle().await?;
            *memory_manager = Some(Arc::new(MemoryManager::new(process_handle)));
        }

        Ok(Arc::clone(memory_manager.as_ref().unwrap()))
    }

    /// Returns the exported functions for a module in the process
    ///
    /// Since it uses `MemoryManager`, the function has to get a process handle with PROCESS_ALL_ACCESS rights.
    #[cfg(windows)]
    pub(crate) async fn get_exports_for_module(
        &self,
        module: &Module,
    ) -> Result<Vec<ExportedFunction>> {
        use windows::Win32::System::Diagnostics::Debug::IMAGE_NT_HEADERS64;
        use windows::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY};

        use crate::InjectError;

        let memory_manager = self.get_memory_manager().await?;

        let mut dos_header = IMAGE_DOS_HEADER::default();
        memory_manager
            .read_from_address(module.load_address, &mut dos_header)
            .await?;

        let mut nt_headers = IMAGE_NT_HEADERS64::default();
        let nt_headers_address = module.load_address + dos_header.e_lfanew as usize;

        memory_manager
            .read_from_address(nt_headers_address, &mut nt_headers)
            .await?;

        let exports_table_address = module.load_address
            + nt_headers.OptionalHeader.DataDirectory[0].VirtualAddress as usize;

        let mut exports = IMAGE_EXPORT_DIRECTORY::default();
        memory_manager
            .read_from_address(exports_table_address, &mut exports)
            .await?;

        let base = exports.Base as u16;
        let functions_address = module.load_address + exports.AddressOfFunctions as usize;
        let names_address = module.load_address + exports.AddressOfNames as usize;
        let names_ordinals_map_address =
            module.load_address + exports.AddressOfNameOrdinals as usize;

        let exported_functions = (0..exports.NumberOfNames as usize).map(|i| {
            let memory_manager_iter = Arc::clone(&memory_manager);
            async move {
                let function_name_addr = names_address + i * 4;
                let function_name_ordinal_map_address = names_ordinals_map_address + i * 2;
                let mut function_name_rva: u32 = 0;
                memory_manager_iter
                    .read_from_address(function_name_addr, &mut function_name_rva)
                    .await?;

                let function_name = memory_manager_iter
                    .read_string_from_char_ptr(module.load_address + function_name_rva as usize)
                    .await?;

                let mut function_ordinal = 0_u16;
                memory_manager_iter
                    .read_from_address(function_name_ordinal_map_address, &mut function_ordinal)
                    .await?;

                let mut function_address_rva = 0_u32;
                memory_manager_iter
                    .read_from_address(
                        functions_address + function_ordinal as usize * 4,
                        &mut function_address_rva,
                    )
                    .await?;
                let function_address = module.load_address + function_address_rva as usize;

                Ok::<ExportedFunction, InjectError>(ExportedFunction {
                    ordinal: (function_ordinal + base) as u32,
                    name: function_name,
                    address: function_address,
                })
            }
        });

        join_all(exported_functions).await.into_iter().collect()
    }

    pub(crate) async fn execute(&self, start_address: usize) -> Result<()> {
        use windows::Win32::Foundation::WAIT_OBJECT_0;
        use windows::Win32::System::Threading::CreateRemoteThread;
        use windows::Win32::System::Threading::WaitForSingleObject;
        use windows::Win32::System::WindowsProgramming::INFINITE;

        unsafe {
            let start_address =
                std::mem::transmute::<usize, extern "system" fn(*mut c_void) -> u32>(start_address);

            let handle = self.get_process_handle().await?.handle;
            let thread_handle: HandleWrapper =
                CreateRemoteThread(handle, None, 0, Some(start_address), None, 0, None)?.into();

            let result = WaitForSingleObject(*thread_handle, INFINITE);
            if result != WAIT_OBJECT_0 {
                Err("Waiting on the created thread failed".into())
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
pub(crate) struct Modules {
    first: bool,
    snapshot_handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Stream for Modules {
    type Item = Module;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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
                Poll::Ready(None)
            } else {
                Poll::Ready(Some(Module {
                    name: utils::get_module_name(&module_entry.szModule),
                    load_address: module_entry.modBaseAddr as usize,
                }))
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
                Poll::Ready(None)
            } else {
                Poll::Ready(Some(Module {
                    name: utils::get_module_name(&module_entry.szModule),
                    load_address: module_entry.modBaseAddr as usize,
                }))
            }
        }
    }
}

pub(crate) struct Module {
    name: String,
    load_address: usize,
}

impl Module {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(windows)]
pub(crate) struct MemoryManager {
    process_handle: Arc<HandleWrapper>,
    /// Addresses and sizes of the allocations
    allocations: Mutex<
        Vec<(
            usize, /* allocation_address */
            usize, /* allocation_size */
        )>,
    >,
}

#[cfg(windows)]
impl MemoryManager {
    pub(crate) fn new(process_handle: Arc<HandleWrapper>) -> MemoryManager {
        MemoryManager {
            process_handle,
            allocations: Mutex::new(vec![]),
        }
    }

    pub(crate) async fn read_from_address<T>(&self, address: usize, buffer: &mut T) -> Result<()> {
        use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

        let handle = self.process_handle.handle;
        assert!(!handle.is_invalid());

        let ret;
        unsafe {
            ret = ReadProcessMemory(
                handle,
                address as *const c_void,
                buffer as *mut T as *mut c_void,
                std::mem::size_of::<T>(),
                None,
            );
        }
        if !ret.as_bool() {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }

    pub(crate) async fn allocate_and_write(&self, data: &[u8]) -> Result<usize> {
        use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
        use windows::Win32::System::Memory::VirtualAllocEx;
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};

        let handle = self.process_handle.handle;

        assert!(!handle.is_invalid());

        let allocated_address = unsafe {
            let allocated_address =
                VirtualAllocEx(handle, None, data.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);

            if allocated_address.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }

            let ret = WriteProcessMemory(
                handle,
                allocated_address,
                data.as_ptr() as *const c_void,
                data.len(),
                None,
            );

            if !ret.as_bool() {
                return Err(std::io::Error::last_os_error().into());
            }

            allocated_address as usize
        };

        let mut allocations = self.allocations.lock().await;
        (*allocations).push((allocated_address, data.len()));
        Ok(allocated_address)
    }

    pub(crate) async fn read_string_from_char_ptr(&self, address: usize) -> Result<String> {
        let mut s: Vec<u8> = vec![];
        let mut read_terminator = false;
        let mut offset = 0_usize;
        let mut buffer = [0_u8; std::mem::size_of::<usize>()];

        loop {
            self.read_from_address(address + offset, &mut buffer)
                .await?;

            s.extend(buffer.iter().take_while(|&&c| {
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

        let s = String::from_utf8(s)?;
        Ok(s)
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        async_std::task::block_on(async {
            use windows::Win32::System::Memory::VirtualFreeEx;
            use windows::Win32::System::Memory::MEM_DECOMMIT;

            let allocations = self.allocations.lock().await;
            for allocation in allocations.iter() {
                let handle = self.process_handle.handle;
                assert!(!handle.is_invalid());
                let allocation_address = (allocation.0) as *mut c_void;
                let allocation_size = allocation.1;
                let ret;
                unsafe {
                    ret = VirtualFreeEx(handle, allocation_address, allocation_size, MEM_DECOMMIT);
                }
                if !ret.as_bool() {
                    panic!("Uh oh something went wrong and we are leaking");
                }
            }
        });
    }
}

#[derive(Debug)]
pub(crate) struct ExportedFunction {
    ordinal: u32,
    name: String,
    address: usize,
}

impl ExportedFunction {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn address(&self) -> usize {
        self.address
    }
}

impl std::fmt::Display for ExportedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({:#02X})", self.ordinal, self.name, self.address)
    }
}

#[cfg(windows)]
pub(crate) struct HandleWrapper {
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
