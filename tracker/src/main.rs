use tracker::Tracker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args();
    let tracker = Tracker::new(args)?;
    tracker.run()?;
    Ok(())
}
