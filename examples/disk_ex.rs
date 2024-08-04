use pakrelly::disk::DiskManager;

fn main() -> std::io::Result<()> {
    let mut disk_manager = DiskManager::open("helloworld")?;
    println!("{:?}", disk_manager);
    let _ = disk_manager.allocate_page();
    let page_id = disk_manager.allocate_page();
    // 2回allocate_pageしているため、4097bytes目から書き込まれる(切り上げられたspaceはNULL)
    disk_manager.write_page_data(page_id, b"Hello world")?;
    Ok(())
}
