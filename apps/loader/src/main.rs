#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;

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

    println!("Load {app_num} app to payload...");

    let mut app_start = PLASH_START + ptr_len;
    (0..app_num).for_each(|i| {
        let app_header = image_header.load_app(app_start);
        app_start = app_header.start + app_header.size;

        println!("App_{i} start: {:#x} size: {} content: {:?}", app_header.start, app_header.size, app_header.content);
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
        AppHeader::new(app_start, app_size, self.read_bytes(app_start, app_size))
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
