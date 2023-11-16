#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::{println, process::exit};

const PLASH_START: usize = 0x22000000;

// app running aspace
// SBI(0x80000000) -> App <- Kernel(0x80200000)
// 0xffff_ffc0_0000_0000
const RUN_START: usize = 0xffff_ffc0_8010_0000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_terminate(exit_code: i32) {
    println!("[ABI:Terminate] Terminate Apps by exit_code: {exit_code}!");
    exit(exit_code);
}

fn abi_entry(abi_num: usize, arg0: usize) {
    match abi_num  {
        SYS_HELLO => abi_hello(),
        SYS_PUTCHAR => abi_putchar(arg0 as u8 as char),
        SYS_TERMINATE => abi_terminate(arg0 as i32),
        _ => panic!("[ABI:Unknown] Unknown ABI: {abi_num}")
    }
}

struct ImageHeader{
    ptr_len: usize
}

struct AppHeader {
    start: usize,
    size: usize,
    content: &'static [u8],
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let ptr_len = 2;

    let image_header = ImageHeader::new(ptr_len);

    let app_num = image_header.load_app_num(PLASH_START);

    println!("Load {app_num} app to payload...\n");

    let mut app_start = PLASH_START + ptr_len;
    (0..app_num).for_each(|i| {
        let app_header = image_header.load_app(app_start);
        app_start = app_header.start + app_header.size;

        println!("App_{i} start: {:#x} size: {} content: {:?}", app_header.start, app_header.size, app_header.content);

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_header.size)
        };
        run_code.copy_from_slice(app_header.content);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("Execute App_{i} ...");
        // execute app
        unsafe { core::arch::asm!("
            la      a0, {abi_entry}
            li      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
            abi_entry = sym abi_entry,
        )}
        println!("Execute App_{i} done\n");
    });

    println!("Load {app_num} app to payload ok!");
}

impl ImageHeader {
    pub fn new(ptr_len: usize) -> Self {
        Self {
            ptr_len
        }
    }

    #[inline]
    pub fn load_app_num(&self, image_start: usize) -> usize {
        self.load_app_size(image_start)
    }

    #[inline]
    pub fn load_app(&self, app_start: usize) -> AppHeader {
        let app_size = self.load_app_size(app_start);
        let app_start = app_start  + self.ptr_len;
        let app_content = self.read_bytes(app_start, app_size);
        AppHeader::new(app_start, app_size, app_content)
    }

    #[inline]
    fn load_app_size(&self, app_start: usize) -> usize {
        let app_size = self.read_bytes(app_start, self.ptr_len);
        self.bytes_to_usize(app_size)
    }
    
    #[inline]
    fn read_bytes(&self, ptr: usize, ptr_len: usize) -> &'static [u8] {
        unsafe { core::slice::from_raw_parts(ptr as *const u8, ptr_len) }
    }
    
    #[inline]
    fn bytes_to_usize(&self, binary: &[u8]) -> usize {  
        let high_byte = binary[0] as usize;
        let low_byte = binary[1] as usize;
        (high_byte << 8) | low_byte
    }
}

impl AppHeader {
    pub fn new(start: usize, size: usize, content: &'static [u8]) -> Self {
        Self {
            start,
            size,
            content,
        }
    }
}
