use anyhow::{Ok, Result};

use crate::btree::BTree;
use crate::tuple;
use crate::{buffer::BufferPoolManager, disk::PageId};

#[derive(Debug)]
pub struct SimpleTable {
    // B+Treeのmeta pageのid
    pub meta_page_id: PageId,
    // 左からいくつの列がprimary keyなのか(複合キーにも対応)
    // 1列だけprimary keyなのであれば "num_key_elems: 1" になる
    pub num_key_elems: usize,
}

impl SimpleTable {
    pub fn create(&mut self, buffer_pool_manager: &mut BufferPoolManager) -> Result<()> {
        let btree = BTree::create(buffer_pool_manager)?;
        self.meta_page_id = btree.meta_page_id;

        Ok(())
    }
    pub fn insert(
        &self,
        buffer_pool_manager: &mut BufferPoolManager,
        record: &[&[u8]],
    ) -> Result<()> {
        let btree = BTree::new(self.meta_page_id);

        // byte列としてのrecordからkeyを抽出
        let mut key = vec![];
        tuple::encode(record[..self.num_key_elems].iter(), &mut key);
        // byte列としてのrecordからvalueを抽出
        let mut value = vec![];
        tuple::encode(record[self.num_key_elems..].iter(), &mut value);

        btree.insert(buffer_pool_manager, &key, &value)?;
        Ok(())
    }
}
