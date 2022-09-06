use injector::assembler::Assembler;
use injector::process::{processes, Module, Process};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mtga_process = find_mtga_process()?;
    let mono_module = find_mono_module(&mtga_process)?;
    let exports = mtga_process.get_exports_for_module(&mono_module)?;
    exports
        .iter()
        .take(5)
        .for_each(|exported_fn| println!("{}", exported_fn));
    // let memory_manager = mtga_process.get_memory_manager()?;

    let mut assembler = Assembler::new();

    // Shadow space on the stack,
    // see [https://stackoverflow.com/questions/30190132/what-is-the-shadow-space-in-x64-assembly]
    assembler.sub_rsp(40);
    assembler.mov_rax(0xdeadc0de);
    assembler.call_rax();
    assembler.add_rsp(40);
    println!("{:#X?}", assembler.data());
    // memory_manager.allocate_and_write(code);
    Ok(())
}

fn find_mtga_process() -> Result<Process, Box<dyn std::error::Error>> {
    match processes().find(|process| process.name() == "MTGA.exe") {
        Some(process) => {
            println!("MTGA.exe ({}) found", process.id());
            Ok(process)
        }
        None => Err("Could not find MTGA.exe")?,
    }
}

fn find_mono_module(process: &Process) -> Result<Module, Box<dyn std::error::Error>> {
    match process
        .modules()
        .find(|module| module.name().starts_with("mono-"))
    {
        Some(module) => Ok(module),
        None => Err("Could not find the mono module")?,
    }
}
