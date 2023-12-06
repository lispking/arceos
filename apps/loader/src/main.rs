#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
extern crate axstd as std;

#[cfg(feature = "axstd")]
use axstd::println;

use elf::{ElfBytes, endian::AnyEndian, abi::PT_LOAD, segment::ProgramHeader};

use std::vec::Vec;
use memory_addr::{align_up_4k, align_down_4k};

const PAGE_SHIFT: usize = 12;
/// The size of a 4K page (4096 bytes).
const PAGE_SIZE_4K: usize = 0x1000;

const PFLASH_START: usize = 0xffff000004000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let apps_start = PFLASH_START as *const u8;
    let load_size = 5116832; // Dangerous!!! We need to get accurate size of apps.
    println!("Load payload ...");
    let load_code = unsafe { core::slice::from_raw_parts(apps_start, load_size)};
    println!("content: {:#x}", bytes_to_usize(&load_code));

    println!("code address [{:?}] {:?}", load_code.as_ptr(), &load_code[..100]);

    match ElfBytes::<AnyEndian>::minimal_parse(&load_code) {
        Ok(file) => {
            println!("ELF parse ok!");

            let phdrs: Vec<ProgramHeader> = file.segments().unwrap()
                .iter()
                .filter(|phdr| phdr.p_type == PT_LOAD)
                .collect();

            let mut end = 0;
            println!("There are {} PT LOAD segments", phdrs.len());
            for phdr in phdrs {
                println!("phdr: offset: {:#X}=>{:#X} size: {:#X}=>{:#X}", phdr.p_offset, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz);
                
                let fdata = file.segment_data(&phdr).unwrap();
                println!("fdata: {:#x}", fdata.len());

                let va_end = align_up_4k((phdr.p_vaddr + phdr.p_memsz) as usize);
                let va = align_down_4k(phdr.p_vaddr as usize);
                let num_pages = (va_end - va) >> PAGE_SHIFT;
                let pa = vm::alloc_pages(num_pages, PAGE_SIZE_4K);
                println!("va: {:#x} pa: {:#x} num {}", va, pa, num_pages);

                vm::map_region(va, pa, num_pages << PAGE_SHIFT);
                let mdata = unsafe {
                    core::slice::from_raw_parts_mut(phdr.p_vaddr as *mut u8, phdr.p_filesz as usize)
                };
                mdata.copy_from_slice(fdata);
                println!("mdata: {:#x}", mdata.len());
                
                if phdr.p_memsz != phdr.p_filesz {
                    let edata = unsafe {
                        core::slice::from_raw_parts_mut((phdr.p_vaddr + phdr.p_filesz) as *mut u8, (phdr.p_memsz - phdr.p_filesz) as usize)
                    };
                    edata.fill(0);
                    println!("edata: {:#x}", edata.len());
                }
            }


            println!("Execute app ...");
            // execute app
            // unsafe { core::arch::asm!("
            //     ldr x8, =0xffff000004400200
            //     blr x8
            //     b   .",
            // )}
            println!("Execute app done.");
        }
        error => {
            println!("ELF parse failed by error {:?}", error);
        }
    }

    println!("Load payload ok!");
}

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    let bytes = &bytes[..8];
    usize::from_be_bytes(bytes.try_into().unwrap())
}
