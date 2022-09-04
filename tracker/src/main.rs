use injector::process::processes;

fn main() {
    if let Some(process) = processes().find(|process| process.name() == "MTGA.exe") {
        println!("Found {} with id {}", process.name(), process.id());
    } else {
        println!("Could not find MTGA.exe");
    }
}
