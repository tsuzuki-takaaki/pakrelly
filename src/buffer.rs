use std::ops::{Index, IndexMut};
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
    next_victim_id: BufferId, // linked-list
}

impl Index<BufferId> for BufferPool {
    type Output = Frame;
    fn index(&self, index: BufferId) -> &Self::Output {
        &self.buffers[index.0]
    }
}

impl IndexMut<BufferId> for BufferPool {
    fn index_mut(&mut self, index: BufferId) -> &mut Self::Output {
        &mut self.buffers[index.0]
    }
}

// どれだけのbufferをpoolingするかは、initialize(BufferPool::new)時に引数で渡す
impl BufferPool {
    fn size(&self) -> usize {
        self.buffers.len()
    }
    // 捨てるbufferを決めて、そのBufferIdを返す
    // BufferPoolで管理しているFrameたち(linked-listで繋がっている)をなめて、精査する
    fn evict(&mut self) -> Option<BufferId> {
        // initialize時に注入される、poolingの最大値
        let pool_size = self.size();
        // poolingしてるbufferの貸し出し数管理
        let mut consecutive_pinned = 0;
        let victim_id = loop {
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];

            // 特定のBufferのcounterが0であればそのBufferIdを返して終了
            if frame.usage_count == 0 {
                break self.next_victim_id;
            }

            // Rc::get_mutは、他に参照が存在する場合にNoneを返す
            // 実行時に参照されていない場合は、usege_countをdecrement
            if Rc::get_mut(&mut frame.buffer).is_some() {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            } else {
                // 参照があった際の処理
                consecutive_pinned += 1;
                if consecutive_pinned >= pool_size {
                    return None;
                }
            }
            self.next_victim_id = self.increment_id(self.next_victim_id)
        };
        Some(victim_id)
    }
    fn increment_id(&self, buffer_id: BufferId) -> BufferId {
        BufferId((buffer_id.0 + 1) % self.size())
    }
}
// poolを見に行って、あれば使い、なければDiskManager越しにDisk I/O
pub struct BufferPoolManager {
    disk: DiskManager,
    pool: BufferPool,
    page_table: HashMap<PageId, BufferId>, // どのpageデータがどのbufferに入っているかをmapping
}
