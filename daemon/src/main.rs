use std::error::Error;
use wayland::init::WaylandContext;

fn main() -> Result<(), Box<dyn Error>> {
    let _wayland = WaylandContext::new()?;

    Ok(())
}
