use std::{fs::File, io::Write};

fn main() -> std::io::Result<()> {
    let mut file = File::create("example.txt")?;
    file.write_all(b"Hello World\n")?;
    Ok(())
}
