//! Handles the memory management specific functionality of UEFI.

use bitflags::bitflags;
use core::{
    mem::size_of,
    slice::{self, Chunks, ChunksMut},
};

use crate::{boot::BootServices, status::Error};

/// Represents a physical address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalAddress(pub u64);

/// Represents a virtual address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VirtualAddress(pub u64);

/// Describes the different areas of memory in the memory map.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of the memory region.
    pub Type: MemoryType,
    /// Physical address of the first byte in the memory region.
    /// PhysicalStart must be aligned on a 4 KiB boundary, and must
    /// not be above 0xfffffffffffff000.
    pub PhysicalStart: PhysicalAddress,
    /// Virtual address of the first byte in the memory region.
    /// VirtualStart must be aligned on a 4 KiB boundary, and must not
    /// be above 0xfffffffffffff000.
    pub VirtualStart: VirtualAddress,
    /// Number of 4 KiB pages in the memory region.
    /// NumberOfPages must not be 0, and must not be any value that
    /// would represent a memory page with a start address, either physical
    /// or virtual, above 0xfffffffffffff000.
    pub NumberOfPages: u64,
    /// Attributes of the memory region that describe the bit mask of
    /// capabilities for that memory region, and not necessarily the current
    /// settings for that memory region.
    pub Attribute: MemoryAttributes,
}

/// Represents the different types memory can have.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MemoryType {
    /// Not usable.
    ReservedMemoryType,
    /// The code portions of a loaded application.
    LoaderCode,
    /// The data portions of a loaded application and the default data allocation
    /// type used by an application to allocate pool memory.
    LoaderData,
    /// The code portions of a loaded Boot Service Driver.
    BootServicesCode,
    /// The data portions of a loaded Boot Serve Driver, and the default data
    /// allocation type used by a Boot Services Driver to allocate pool memory.
    BootServicesData,
    /// The code portions of a loaded Runtime Driver.
    RuntimeServicesCode,
    /// The data portions of a loaded Runtime Driver and the default
    /// data allocation type used by a Runtime Driver to allocate pool memory.
    RuntimeServicesData,
    /// Free (unallocated) memory.
    ConventionalMemory,
    /// Memory in which errors have been detected.
    UnusableMemory,
    /// Memory that holds the ACPI tables.
    ACPIReclaimMemory,
    /// Address space reserved for use by the firmware.
    ACPIMemoryNVS,
    /// Used by system firmware to request that a memory-mapped IO region
    /// be mapped by the OS to a virtual address so it can be accessed by EFI runtime services.
    MemoryMappedIO,
    /// System memory-mapped IO region that is used to translate memory
    /// cycles to IO cycles by the processor.
    MemoryMappedIOPortSpace,
    /// Address space reserved by the firmware for code that is part of the processor.
    PalCode,
    /// A memory region that operates as EfiConventionalMemory,
    /// however it happens to also support byte-addressable non-volatility.
    PersistentMemory,
    MaxMemoryType,
}

bitflags! {
    /// Memory Attribute Definitions
    pub struct MemoryAttributes: u64 {
        /// Memory cacheability attribute: The memory region supports being configured as not cacheable.
        const UC = 0x0000_0000_0000_0001;
        /// Memory cacheability attribute: The memory region supports being configured as write combining.
        const WC = 0x0000_0000_0000_0002;
        /// Memory cacheability attribute: The memory region supports being
        // configured as cacheable with a “write through” policy. Writes that
        // hit in the cache will also be written to main memory.
        const WT = 0x0000_0000_0000_0004;
        /// Memory cacheability attribute: The memory region supports being
        /// configured as cacheable with a “write back” policy. Reads and writes
        /// that hit in the cache do not propagate to main memory. Dirty data is
        /// written back to main memory when a new cache line is allocated.
        const WB = 0x0000_0000_0000_0008;
        /// Memory cacheability attribute: The memory region supports being
        /// configured as not cacheable, exported, and supports the “fetch
        /// and add” semaphore mechanism.
        const UCE = 0x0000_0000_0000_0010;
        /// Physical memory protection attribute: The memory region supports
        /// being configured as write-protected by system hardware. This is
        /// typically used as a cacheability attribute today. The memory region
        /// supports being configured as cacheable with a "write protected"
        /// policy. Reads come from cache lines when possible, and read misses
        /// cause cache fills. Writes are propagated to the system bus and
        /// cause corresponding cache lines on all processors on the bus to be
        /// invalidated.
        const WP = 0x0000_0000_0000_1000;
        /// Physical memory protection attribute: The memory region supports
        /// being configured as read-protected by system hardware.
        const RP = 0x0000_0000_0000_2000;
        /// Physical memory protection attribute: The memory region supports
        /// being configured so it is protected by system hardware from
        /// executing code.
        const XP = 0x0000_0000_0000_4000;
        ///  Runtime memory attribute: The memory region refers to persistent memory.
        const NV = 0x0000_0000_0000_8000;
        /// The memory region provides higher reliability relative to other
        /// memory in the system. If all memory has the same reliability, then
        /// this bit is not used.
        const MORE_RELIABLE = 0x0000_0000_0001_0000;
        /// Physical memory protection attribute: The memory region supports
        /// making this memory range read-only by system hardware.
        const RO = 0x0000_0000_0002_0000;
        /// Runtime memory attribute: The memory region needs to be given a
        /// virtual mapping by the operating system when SetVirtualAddressMap()
        /// is called.
        const RUNTIME = 0x8000_0000_0000_0000;
    }
}

/// Represents a memory map.
#[derive(Debug)]
pub struct MemoryMap {
    /// The buffer where the contents of the memory map are located.
    pub(crate) buffer: *const MemoryDescriptor,
    /// The size, in bytes, of the memory map.
    pub(crate) size: usize,
    /// The key of the memory map.
    ///
    /// This is used to call `ExitBootServices`.
    pub(crate) key: usize,
    /// The size of a single memory descriptor within the `MemoryMap`.
    pub(crate) descriptor_size: usize,
    /// The version of the memory descriptors.
    pub(crate) version: u32,
}

impl MemoryMap {
    /// The amount of entries in the memory map.
    pub fn len(&self) -> usize {
        self.size / self.descriptor_size
    }

    /// Returns true if the memory map does not have eny entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the `MemoryDescriptor`s in the `MemoryMap`.
    pub fn iter(&self) -> MemoryMapIterator {
        // This is safe under the assumption that the buffer has the specified size and is valid.
        let buffer = unsafe { slice::from_raw_parts(self.buffer as *const u8, self.size) };

        MemoryMapIterator {
            iter: buffer.chunks(self.descriptor_size),
            descriptor_size: self.descriptor_size,
            version: self.version,
        }
    }

    /// Returns an iterator over the `MemoryDescriptor`s in the `MemoryMap`.
    pub fn iter_mut(&mut self) -> MemoryMapIteratorMut {
        // This is safe under the assumption that the buffer has the specified size and is valid.
        let buffer = unsafe { slice::from_raw_parts_mut(self.buffer as *mut u8, self.size) };

        MemoryMapIteratorMut {
            iter: buffer.chunks_mut(self.descriptor_size),
            descriptor_size: self.descriptor_size,
            version: self.version,
        }
    }

    /// Drops this memory map, deallocating the underlying buffer.
    ///
    /// # Safety
    /// This function assumes that the boot services are still active:
    /// Make sure not to call it after calling `ExitBootServices`.
    pub unsafe fn drop(self, boot_services: &'static BootServices) -> Result<(), Error> {
        boot_services.free_pool(self.buffer as *const u8)?;

        Ok(())
    }
}

/// An iterator over the memory map entries.
pub struct MemoryMapIterator<'a> {
    /// The buffer where the contents of the memory map are located.
    iter: Chunks<'a, u8>,
    /// The size of a single memory descriptor within the `MemoryMap`.
    descriptor_size: usize,
    /// The version of the memory descriptors.
    version: u32,
}

impl<'a> Iterator for MemoryMapIterator<'a> {
    type Item = &'a MemoryDescriptor;

    fn next(&mut self) -> Option<&'a MemoryDescriptor> {
        if let Some(chunk) = self.iter.next() {
            if chunk.len() == self.descriptor_size {
                debug_assert!(
                    self.descriptor_size >= size_of::<MemoryDescriptor>(),
                    "The size of the memory descriptor is smaller than the standard says."
                );

                // This is safe, because of the assertion above. That condition should hold according to the UEFI specification.
                unsafe { Some(&*(chunk.as_ptr() as *const MemoryDescriptor)) }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// An iterator over the memory map entries.
pub struct MemoryMapIteratorMut<'a> {
    /// The buffer where the contents of the memory map are located.
    iter: ChunksMut<'a, u8>,
    /// The size of a single memory descriptor within the `MemoryMap`.
    descriptor_size: usize,
    /// The version of the memory descriptors.
    version: u32,
}

impl<'a> Iterator for MemoryMapIteratorMut<'a> {
    type Item = &'a mut MemoryDescriptor;

    fn next(&mut self) -> Option<&'a mut MemoryDescriptor> {
        if let Some(chunk) = self.iter.next() {
            if chunk.len() == self.descriptor_size {
                debug_assert!(
                    self.descriptor_size >= size_of::<MemoryDescriptor>(),
                    "The size of the memory descriptor is smaller than the standard says."
                );

                // This is safe, because of the assertion above. That condition should hold according to the UEFI specification.
                unsafe { Some(&mut *(chunk.as_ptr() as *mut MemoryDescriptor)) }
            } else {
                None
            }
        } else {
            None
        }
    }
}
