pub(crate) fn get_process_name(process_name_raw: &[u16]) -> String {
    let len = process_name_raw.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&process_name_raw[..len]).unwrap()
}

pub(crate) fn get_module_name(raw_module_name: &[u16]) -> String {
    let len = raw_module_name.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&raw_module_name[..len]).unwrap()
}

