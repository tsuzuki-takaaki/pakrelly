use std::{
    fs::File,
    io::{Seek, Write},
};

fn main() -> std::io::Result<()> {
    let mut file = File::create("example.txt")?;
    file.seek(std::io::SeekFrom::Start(200))?;
    file.write_all(b"Hello World\n")?;
    Ok(())
}
