use core::arch::asm;

/// Copy `size` bytes from `src` to `dst`.
///
/// # Safety
///
/// This function has all the safety requirements of `core::ptr::copy` except
/// that data races (both on `src` and `dst`) are explicitly permitted.
#[inline(always)]
pub unsafe fn unsafe_copy_bytes<T>(src: *const T, dst: *mut T, count: usize) {
    let size = count * core::mem::size_of::<T>();
    // SAFETY: Inline assembly to perform a memory copy.
    // The safery requirements of the parameters are delegated to the caller of
    // this function which is unsafe.
    unsafe {
        /*
        asm!(
            "rep movsb",
            inout("x4") src => _,
            inout("x3") dst => _,
            inout("x2") size => _,
            options(nostack),
        );
        */
        // ARM64 没有 `rep movsb`，只能使用循环加载/存储。
        // 每次复制 8 字节，最后不足部分逐字节处理。
        asm!(
            // x0 = src, x1 = dst, x2 = size
            "1:",
            "cmp x2, #8",            // 如果剩余 >= 8 字节
            "blt 2f",                // 否则跳到逐字节复制
            "ldr x3, [x0], #8",      // 从 src 读取 8 字节
            "str x3, [x1], #8",      // 写入 dst
            "sub x2, x2, #8",        // 剩余字节数减 8
            "b 1b",
            "2:",
            "cbz x2, 4f",            // 如果没有剩余字节 -> 结束
            "3:",
            "ldrb w3, [x0], #1",     // 从 src 读一个字节
            "strb w3, [x1], #1",     // 写到 dst
            "subs x2, x2, #1",
            "b.ne 3b",
            "4:",
            inout("x0") src => _,
            inout("x1") dst => _,
            inout("x2") size => _,
            out("x3") _,
            options(nostack, preserves_flags),
        );
    }
}

/// Set `size` bytes at `dst` to `val`.
///
/// # Safety
///
/// This function has all the safety requirements of `core::ptr::write_bytes` except
/// that data races are explicitly permitted.
#[inline(always)]
pub unsafe fn write_bytes<T>(dst: *mut T, count: usize, value: u8) {
    let size = count * core::mem::size_of::<T>();
    // SAFETY: Inline assembly to perform a memory write.
    // The safery requirements of the parameters are delegated to the caller of
    // this function which is unsafe.
    unsafe {
        /*
        asm!(
            "rep stosb",
            inout("x3") dst => _,
            inout("x2") size => _,
            in("x0") value,
            options(nostack),
        );
        */
        asm!(
        // x0 = dst, w1 = value, x2 = size
        // 构造一个 64 位重复字节的填充值
            "uxtb x3, w1",              // 零扩展到 64 位
            "orr x3, x3, x3, lsl #8",   // 重复 16bit
            "orr x3, x3, x3, lsl #16",  // 重复 32bit
            "orr x3, x3, x3, lsl #32",  // 重复 64bit

            "1:",
                "cmp x2, #8",
                "blt 2f",
                "str x3, [x0], #8",
                "sub x2, x2, #8",
                "b 1b",
            "2:",
                "cbz x2, 4f",
            "3:",
                "strb w1, [x0], #1",
               "subs x2, x2, #1",
                "b.ne 3b",
            "4:",
            inout("x0") dst => _,
            in("w1") value,
            inout("x2") size => _,
            out("x3") _,
            options(nostack, preserves_flags),
        );
    }
}
