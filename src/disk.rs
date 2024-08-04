use std::{fs::File, io};

// page sizeは4KB(4096)で設定
pub const PAGE_SIZE: usize = 4096;

pub struct DiskManager {
    heap_file: File,
    next_page_id: u64,
}

impl DiskManager {
    pub fn new(heap_file: File) -> io::Result<Self> {
        // file descriptorを受け取り、現状のファイルの書き込まれ具合を取得
        let heap_file_size = heap_file.metadata()?.len();
        // ファイルの状態から、対象のファイルをpageという単位で分けた時に、次に書き込むべきpageはどこなのかを特定(page0, page1, ...etc)
        let next_page_id = heap_file_size / PAGE_SIZE as u64;
        Ok(Self {
            heap_file,
            next_page_id,
        })
    }
}
