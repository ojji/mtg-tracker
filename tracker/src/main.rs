use injector::Mtga;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mtga_process =
        Mtga::new(r"./mtga-datacollector/bin/x64/Release/netstandard2.1/mtga-datacollector.dll")?;
    mtga_process.inject_tracker()?;
    println!("Tracker successfully loaded!");
    Ok(())
}
