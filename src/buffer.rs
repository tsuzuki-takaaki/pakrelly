use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::disk::{DiskManager, PageId, PAGE_SIZE};

pub type Page = [u8; PAGE_SIZE];

#[derive(Default, Clone, Copy)]
pub struct BufferId(usize);
// RefCell: データ競合について、コンパイル時ではなく実行時に検査する
// Cell: 読み取り専用の値の中に書き込み可能な値をつくる
pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: Cell<bool>,
}
// Bufferに対するpointer + metadata(usage_countはそのbufferがどれだけ使われたかを持つ)
// Rc: 対象のデータへの参照の数を実行時にtrackingする. カウントが0になってどこからも利用されてないことがわかったら対象のメモリ領域を自動で解放する
pub struct Frame {
    usage_count: u64,
    buffer: Rc<Buffer>,
}
pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId,
}
pub struct BufferPoolManager {
    disk: DiskManager,
    pool: BufferPool,
    page_table: HashMap<PageId, BufferId>, // どのpageデータがどのbufferに入っているかをmapping
}
