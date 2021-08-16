mod block_allocator;
mod bump_allocator;

use bump_allocator::BumpAllocator;

#[global_allocator]
static GLOBAL_ALLOC: BumpAllocator = BumpAllocator::new();
