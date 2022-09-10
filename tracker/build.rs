use std::process::Command;

pub fn main() {
    Command::new("dotnet")
        .args(&["publish", "../mtga-datacollector", "-c", "Release"])
        .output()
        .expect("Could not build the data collector");
    println!("cargo:rerun-if-changed=../mtga-datacollector/*");
}
