// NOTE TO SELF BEFORE EVER PUTTING ON CRATES.IO: There is probably unsoundness if passing
// references as arguments, because lifetimes will be lost? In which case an even bigger disclaimer
// should probably be put than the previsional “this will make monkeys fly out of your nose” one.

// TODO: could improve this by ensuring alignment and using read/write instead of the _unaligned
// versions
//
// TODO: make it possible to panic out of a reordered expression… maybe? most likely we can do
// better than aborting anyway
//
// TODO: do a make_reorderable! instead, that'd avoid having to write the types of the arguments
// each time?

// We need each OrderToken to have its own address in order to have the right clobbers to add.
// Force that by using an explicit u8, even though it's never actually used.
pub struct OrderToken(u8);

impl OrderToken {
    pub fn new() -> OrderToken {
        OrderToken(0)
    }
}

#[macro_export]
macro_rules! reorderable {
    ($token:expr, $fn:ident :: $($ty:ty),* => $ret:ty, $($arg:ident),* $(,)*) => {{
        unsafe extern "C" fn do_it(args_ptr: *const ($($ty),*), ret_ptr: *mut $ret) {
            let ($($arg),*): ($($ty),*) = unsafe { std::ptr::read_unaligned(args_ptr) };
            let ret = std::panic::catch_unwind(|| $fn($($arg),*));
            match ret {
                Ok(ret) => {
                    unsafe { std::ptr::write_unaligned(ret_ptr, ret) };
                }
                Err(_) => {
                    std::process::abort();
                }
            }
        }

        #[cfg(not(feature = "plz-ub"))]
        let ret = {
            let _ = &$token;
            $fn($($arg),*)
        };

        #[cfg(feature = "plz-ub")]
        let ret = {
            let args: ($($ty),*) = ($($arg),*);
            let mut args_buf = [0u8; std::mem::size_of::<($($ty),*)>()];
            unsafe { std::ptr::write_unaligned(&mut args_buf as *mut _ as *mut _, args) };
            let mut ret_buf = [0u8; std::mem::size_of::<$ret>()];

            let args_ptr = &args_buf;
            let ret_ptr = &mut ret_buf;
            let mut token_ptr: &mut $crate::OrderToken = &mut $token;

            #[cfg(target_arch = "x86_64")]
            unsafe {
                asm!(
                    "call {}",
                    sym do_it,
                    // Actual arguments
                    inout("rdi") args_ptr => _,
                    inout("rsi") ret_ptr => _,
                    // The token, passed as a pseudo bonus argument
                    inout("rdx") token_ptr => _,
                    // All caller-saved registers must be marked as clobberred
                    out("rax") _, out("rcx") _,
                    out("r8") _, out("r9") _, out("r10") _, out("r11") _,
                    out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
                    out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
                    out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
                    out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
                );
            }

            #[cfg(not(target_arch = "x86_64"))]
            std::compile_error!("Asking for UB on non-x86_64… we currently know how to UB properly only on x86_64");

            unsafe { std::ptr::read_unaligned(&ret_buf) }
        };

        ret
    }};
}
