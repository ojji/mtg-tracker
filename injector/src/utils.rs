use rand::RngCore;

pub(crate) fn get_process_name(process_name_raw: &[u16]) -> String {
    let len = process_name_raw.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&process_name_raw[..len]).unwrap()
}

pub(crate) fn get_module_name(raw_module_name: &[u16]) -> String {
    let len = raw_module_name.iter().take_while(|&&c| c != 0).count();
    String::from_utf16(&raw_module_name[..len]).unwrap()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Guid {
    data: [u8; 16],
}

impl Guid {
    pub fn rand() -> Guid {
        let mut rand = rand::thread_rng();
        let mut data = [0_u8; 16];
        rand.fill_bytes(&mut data);

        Guid { data }
    }
}

impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 63b00000-bfde-01d3-7852-290676ece2d7
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:04X}-{:08X}{:04X}",
            u32::from_be_bytes(self.data[0..4].try_into().unwrap()),
            u16::from_be_bytes(self.data[4..6].try_into().unwrap()),
            u16::from_be_bytes(self.data[6..8].try_into().unwrap()),
            u16::from_be_bytes(self.data[8..10].try_into().unwrap()),
            u32::from_be_bytes(self.data[10..14].try_into().unwrap()),
            u16::from_be_bytes(self.data[14..16].try_into().unwrap()),
        )
    }
}
