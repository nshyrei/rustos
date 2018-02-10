use memory::util::frame_bitmap::FrameBitMap;
use memory::allocator::bump::ConstSizeBumpAllocator;
use std::mem;
use std::u8;

#[test]
fn bitmap_new_should_create_empty_bitmap_of_size_zero_if_frame_count_is_inside_bitmap_entry_size() {
    // bitmap entry size = 1 byte (holds 8 entries)
    // test frame size = 3..16 byte
    // available mem size = 16 byte
    // frame count = 16 / test frame size - always lower then 8
    let default_memory_value = 10;
    let bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    for test_frame_size in 3..17 {
        let frames_count = 16 / test_frame_size;
        let bitmap_cell_size = FrameBitMap::cell_size(frames_count);
        let mut KERNEL_BASIC_HEAP_ALLOCATOR = ConstSizeBumpAllocator::from_address(addr, 16, bitmap_cell_size);
        let mut bitmap = FrameBitMap::new_from_available_memory(16, test_frame_size, &mut KERNEL_BASIC_HEAP_ALLOCATOR);

        assert!(bytes[0] == 0,
                "Bitmap entry wasn't created. Memory value at index zero is {} but should be zero, frame size is {}, memory dump {:?}",
                bytes[0],
                test_frame_size,
                bytes);

        for i in 1..16 {
            assert!(bytes[i] == default_memory_value,
                    "Memory at address {} was corrupted. Memory value should be {} but was {}, frame size is {}, memory dump {:?}",
                    i,
                    default_memory_value,
                    bytes[1],
                    test_frame_size,
                    bytes);
        }
    }
}

#[test]
fn bitmap_new_should_create_empty_bitmap_of_size_2_if_frame_count_is_outside_bitmap_entry_size() {
    // bitmap entry size = 1 byte (holds 8 entries)
    // test frame size = 1..2 byte
    // available mem size = 16 byte
    // frame count = 16 / test frame size - always bigger or eq then 8
    let default_memory_value = 10;
    let bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    for test_frame_size in 1..3 {
        let frames_count = 16 / test_frame_size;
        let bitmap_cell_size = FrameBitMap::cell_size(frames_count);
        let mut KERNEL_BASIC_HEAP_ALLOCATOR = ConstSizeBumpAllocator::from_address(addr, 16, bitmap_cell_size);        
        let mut bitmap = FrameBitMap::new_from_available_memory(16, test_frame_size, &mut KERNEL_BASIC_HEAP_ALLOCATOR);

        assert!(bytes[0] == 0 && bytes[1] == 0,
                "Bitmap entries weren't created. The first 2 entries should be zero, frame size is {}, first two entries {}, {}, memory dump {:?}",
                test_frame_size,
                bytes[0],
                bytes[1],
                bytes);

        for i in 2..16 {
            assert!(bytes[i] == default_memory_value,
                    "Memory at address {} was corrupted. Memory value should be {} but was {}, frame size is {}, memory dump {:?}",
                    i,
                    default_memory_value,
                    bytes[1],
                    test_frame_size,
                    bytes);
        }
    }
}

#[test]
fn bitmap_indexer_should_properly_set_in_use() {
    let default_memory_value = 10;
    let bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    let mut KERNEL_BASIC_HEAP_ALLOCATOR = ConstSizeBumpAllocator::from_address(addr, 16, 1);
    // frame size = 1 byte
    // available memory = 16 byte
    // bitmap entry holds 8 frame entries
    // 2 bitmap entries should be created
    let mut bitmap = FrameBitMap::new_from_available_memory(16, 1 , &mut KERNEL_BASIC_HEAP_ALLOCATOR);

    for i in 0..16 {
        bitmap.set_in_use(i);
    }

    // all two entries should contain only 1s, thus resulting in u8::max_value()
    assert!(bytes[0] == u8::MAX && bytes[1] == u8::MAX,
            "Not all bits were set to 1 in first 2 entries: {:b}, {:b}. Memory dump {:?}",
            bytes[0],
            bytes[1],
            bytes);
}

#[test]
fn bitmap_indexer_should_properly_clear_in_use() {
    let default_memory_value = 10;
    let mut bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    let mut KERNEL_BASIC_HEAP_ALLOCATOR = ConstSizeBumpAllocator::from_address(addr, 16, 1);
    // frame size = 1 byte
    // available memory = 16 byte
    // bitmap entry holds 8 frame entries
    // 2 bitmap entries should be created
    let mut bitmap = FrameBitMap::new_from_available_memory(16, 1, &mut KERNEL_BASIC_HEAP_ALLOCATOR);

    //all 1s
    bytes[0] = u8::MAX;
    bytes[1] = u8::MAX;

    for i in 0..16 {
        bitmap.set_free(i);
    }

    assert!(bytes[0] == 0 && bytes[1] == 0,
            "Not all bits were set to 0 in first 2 entries: {:b}, {:b}. Memory dump {:?}",
            bytes[0],
            bytes[1],
            bytes);
}

#[test]
fn bitmap_indexer_should_properly_test_in_use() {
    let default_memory_value = 10;
    let mut bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    let mut KERNEL_BASIC_HEAP_ALLOCATOR = ConstSizeBumpAllocator::from_address(addr, 16, 1);
    // frame size = 1 byte
    // available memory = 16 byte
    // bitmap entry holds 8 frame entries
    // 2 bitmap entries should be created
    let mut bitmap = FrameBitMap::new_from_available_memory(16, 1, &mut KERNEL_BASIC_HEAP_ALLOCATOR);

    for i in 0..16 {
        bitmap.set_in_use(i);
        assert!(bitmap.is_in_use(i),
                "Entry for frame number {} wasn't set to used. First two entries {}, {}. Memory dump {:?}",
                i,
                bytes[0],
                bytes[1],
                bytes);
    }

    for i in 0..16 {
        bitmap.set_free(i);
        assert!(!bitmap.is_in_use(i),
                "Entry for frame number {} wasn't set to free. First two entries {}, {}. Memory dump {:?}",
                i,
                bytes[0],
                bytes[1],
                bytes);
    }
}