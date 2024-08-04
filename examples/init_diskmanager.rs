use pakrelly::disk::DiskManager;

fn main() -> std::io::Result<()> {
    let disk_manager = DiskManager::open("helloworld")?;
    println!("{:?}", disk_manager);
    // DiskManager { heap_file: File { fd: 3, path: "/Users/tsuzuki-takaaki/sandbox/pakrelly/helloworld", read: true, write: true }, next_page_id: 0 }
    Ok(())
}
