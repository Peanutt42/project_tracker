use std::{env, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
	// this sets the icon of the bin to icon-handdraw.ico for windows
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("assets/icon-handdrawn.ico")
            .compile()?;
    }
    Ok(())
}