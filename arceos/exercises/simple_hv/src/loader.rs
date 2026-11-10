use std::fs::File;
use std::io::{self, Read};

use alloc::vec::Vec;

use axhal::mem::{VirtAddr, PAGE_SIZE_4K};
use axhal::paging::MappingFlags;
use axmm::AddrSpace;

use crate::VM_ENTRY;

/// Address at which the guest expects its boot argument.
///
/// `skernel2` starts with `ld a0, 64(zero)`, i.e. it reads a word from
/// guest-physical address `0x40` before trapping into the hypervisor.
const GUEST_BOOT_ARG_ADDR: usize = 0x40;
/// Value the guest must observe at [`GUEST_BOOT_ARG_ADDR`].
const GUEST_BOOT_ARG_VALUE: usize = 0x6688;

pub fn load_vm_image(fname: &str, uspace: &mut AddrSpace) -> io::Result<()> {
    let mut file = File::open(fname)?;
    let mut image = Vec::new();
    file.read_to_end(&mut image)?;
    ax_println!("guest image: {} bytes", image.len());

    // Map enough pages to hold the whole image, then copy it in.
    let image_size = (image.len() + PAGE_SIZE_4K - 1) & !(PAGE_SIZE_4K - 1);
    uspace
        .map_alloc(
            VM_ENTRY.into(),
            image_size,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
            true,
        )
        .expect("failed to map guest image");
    uspace
        .write(VM_ENTRY.into(), &image)
        .expect("failed to copy guest image");

    // Give the guest the low page that holds its boot argument. The guest runs
    // with `satp` cleared, so guest virtual addresses are guest physical ones.
    uspace
        .map_alloc(
            VirtAddr::from_usize(0),
            PAGE_SIZE_4K,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            true,
        )
        .expect("failed to map guest boot page");
    uspace
        .write(
            VirtAddr::from_usize(GUEST_BOOT_ARG_ADDR),
            &GUEST_BOOT_ARG_VALUE.to_le_bytes(),
        )
        .expect("failed to write guest boot argument");

    Ok(())
}
