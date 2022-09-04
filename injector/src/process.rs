#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
pub fn processes() -> Processes {
    use std::os::windows::prelude::{HandleOrInvalid, OwnedHandle};
    use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS};

    let handle;
    unsafe {
        handle = OwnedHandle::try_from(HandleOrInvalid::from_raw_handle(CreateToolhelp32Snapshot(
            TH32CS_SNAPPROCESS,
            0,
        )));
    }

    Processes {
        first: true,
        snapshot_handle: handle.unwrap(),
    }
}

#[cfg(windows)]
pub struct Processes {
    first: bool,
    snapshot_handle: std::os::windows::prelude::OwnedHandle,
}

impl Iterator for Processes {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        use std::os::windows::prelude::AsRawHandle;
        use winapi::shared::minwindef::{FALSE, MAX_PATH};
        use winapi::um::tlhelp32::{Process32FirstW, Process32NextW, PROCESSENTRY32W};

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; MAX_PATH],
        };

        if self.first {
            self.first = false;
            let ret;
            unsafe {
                ret = Process32FirstW(
                    self.snapshot_handle.as_raw_handle(),
                    &mut process_entry as *mut PROCESSENTRY32W,
                );
            }
            if ret == FALSE {
                None
            } else {
                Some(Process {
                    id: process_entry.th32ProcessID,
                    name: get_process_name(&process_entry.szExeFile),
                })
            }
        } else {
            let ret;
            unsafe {
                ret = Process32NextW(
                    self.snapshot_handle.as_raw_handle(),
                    &mut process_entry as *mut PROCESSENTRY32W,
                );
            }
            if ret == FALSE {
                None
            } else {
                Some(Process {
                    id: process_entry.th32ProcessID,
                    name: get_process_name(&process_entry.szExeFile),
                })
            }
        }
    }
}

fn get_process_name(process_name_raw: &[u16]) -> String {
    let len = process_name_raw.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&process_name_raw[..len]).unwrap()
}

pub struct Process {
    id: u32,
    name: String,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
