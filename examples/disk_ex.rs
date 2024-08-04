use pakrelly::disk::{DiskManager, PAGE_SIZE};

fn main() -> std::io::Result<()> {
    let mut disk_manager = DiskManager::open("helloworld")?;
    let page_id = disk_manager.allocate_page();

    // Write(常にpage単位で書き込み)
    let mut buf = Vec::with_capacity(PAGE_SIZE);
    buf.extend_from_slice(b"Hello World!");
    buf.resize(PAGE_SIZE, 0);
    disk_manager.write_page_data(page_id, &buf)?;

    // Read(常にpage単位で読み込み)
    let mut buf = vec![0; PAGE_SIZE];
    disk_manager.read_page_data(page_id, &mut buf)?;

    println!("{:?}", buf);
    Ok(())
}
