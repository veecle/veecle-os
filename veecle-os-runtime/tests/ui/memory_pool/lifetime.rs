use veecle_os_runtime::memory_pool::MemoryPool;

fn main() {
    let _chunk = {
        let pool: MemoryPool<usize, 2> = MemoryPool::new();
        pool.chunk(0)
    };
}
