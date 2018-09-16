//! UEFI uses the EFI Boot Services Table, which contains a table header and pointers to all of the boot
//! services. The definition for this table is shown in the following code fragments. Except for the table
//! header, all elements in the EFI Boot Services Tables are prototypes of function pointers to functions
//! as defined in Section 7. The function pointers in this table are not valid after the operating system
//! has taken control of the platform with a call to EFI_BOOT_SERVICES.ExitBootServices().

use core::mem::size_of;

use crate::{
    guid::Guid,
    memory::{MemoryDescriptor, MemoryMap, MemoryType, PAGE_SIZE, PhysicalAddress},
    status::{Error, Status, SUCCESS},
    Event, Handle, TableHeader,
};

/// Indicates whether Interface is supplied in native form.
#[repr(C)]
pub enum InterfaceType {
    /// Interface is supplied in native form.
    Native,
}

/// Specifies which handle(s) are to be returned.
#[repr(C)]
pub enum LocateSearchType {
    /// Retrieve all the handles in the handle database.
    AllHandles,
    /// SearchKey supplies the Registration value returned by
    /// EFI_BOOT_SERVICES.RegisterProtocolNotify(). The
    /// function returns the next handle that is new for the registration.
    /// Only one handle is returned at a time, starting with the first, and the
    /// caller must loop until no more handles are returned. Protocol is
    /// ignored for this search type.
    ByRegisterNotify,
    /// All handles that support Protocol are returned. SearchKey is ignored
    /// for this search type.
    ByProtocol,
}

/// Contains a table header and pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    /// The table header for the EFI Boot Services Table. This header
    /// contains the EFI_BOOT_SERVICES_SIGNATURE and
    /// EFI_BOOT_SERVICES_REVISION values along with the size of
    /// the EFI_BOOT_SERVICES structure and a 32-bit CRC to verify
    /// that the contents of the EFI Boot Services Table are valid.
    pub Hdr: TableHeader,
    /// Raises the task priority level.
    RaiseTpl: extern "win64" fn(NewTpl: usize) -> usize,
    /// Restores/lowers the task priority level.
    RestoreTpl: extern "win64" fn(OldTpl: usize),
    /// Allocates pages of a particular type.
    AllocatePages: extern "win64" fn(
        AllocType: usize,
        MemoryType: MemoryType,
        Pages: usize,
        Memory: &mut PhysicalAddress,
    ) -> Status,
    /// Frees allocated pages.
    FreePages: extern "win64" fn(Memory: PhysicalAddress, Pages: usize) -> Status,
    /// Returns the current boot services memory map and memory map key.
    GetMemoryMap: extern "win64" fn(
        MemoryMapSize: &mut usize,
        MemoryMap: *mut MemoryDescriptor,
        MapKey: &mut usize,
        DescriptorSize: &mut usize,
        DescriptorVersion: &mut u32,
    ) -> Status,
    /// Allocates a pool of a particular type.
    AllocatePool:
        extern "win64" fn(PoolType: MemoryType, Size: usize, Buffer: &mut usize) -> Status,
    /// Frees allocated pool.
    FreePool: extern "win64" fn(Buffer: usize) -> Status,
    /// Creates a general-purpose event structure.
    CreateEvent: extern "win64" fn(),
    /// Sets an event to be signaled at a particular time.
    SetTimer: extern "win64" fn(),
    /// Stops execution until an event is signaled.
    WaitForEvent:
        extern "win64" fn(NumberOfEvents: usize, Event: *const Event, Index: &mut usize) -> Status,
    /// Signals an event.
    SignalEvent: extern "win64" fn(),
    /// Closes and frees an event structure.
    CloseEvent: extern "win64" fn(),
    /// Checks whether an event is in the signaled state.
    CheckEvent: extern "win64" fn(),
    /// Installs a protocol interface on a device handle.
    InstallProtocolInterface: extern "win64" fn(
        Handle: &mut Handle,
        Protocol: &Guid,
        InterfaceType: InterfaceType,
        Interface: usize,
    ) -> Status,
    /// Reinstalls a protocol interface on a device handle.
    ReinstallProtocolInterface: extern "win64" fn(),
    /// Removes a protocol interface from a device handle.
    UninstallProtocolInterface:
        extern "win64" fn(Handle: Handle, Protocol: &Guid, Interface: usize) -> Status,
    /// Queries a handle to determine if it supports a specified protocol.
    HandleProtocol:
        extern "win64" fn(Handle: Handle, Protocol: &Guid, Interface: &mut usize) -> Status,
    /// Reserved. Must be NULL.
    _rsvd: usize,
    /// Registers an event that is to be signaled whenever an interface is
    /// installed for a specified protocol.
    RegisterProtocolNotify: extern "win64" fn(),
    /// Returns an array of handles that support a specified protocol.
    LocateHandle: extern "win64" fn(
        SearchType: LocateSearchType,
        Protocol: &Guid,
        SearchKey: usize,
        BufferSize: &mut usize,
        Buffer: *mut Handle,
    ) -> Status,
    /// Locates all devices on a device path that support a specified
    /// protocol and returns the handle to the device that is closest to
    /// the path.
    LocateDevicePath: extern "win64" fn(),
    /// Adds, updates, or removes a configuration table from the EFI
    /// System Table.
    InstallConfigurationTable: extern "win64" fn(),
    /// Loads an EFI image into memory.
    LoadImage: extern "win64" fn(
        BootPolicy: bool,
        ParentImageHandle: Handle,
        DevicePath: usize, /*TODO*/
        SourceBuffer: *const u8,
        SourceSize: usize,
        ImageHandle: &mut Handle,
    ) -> Status,
    /// Transfers control to a loaded image’s entry point.
    StartImage:
        extern "win64" fn(ImageHandle: Handle, ExitDataSize: &mut usize, ExitData: &mut *mut u16)
            -> Status,
    /// Exits the image’s entry point.
    Exit: extern "win64" fn(
        ImageHandle: Handle,
        ExitStatus: isize,
        ExitDataSize: usize,
        ExitData: *const u16,
    ) -> Status,
    /// Unloads an image.
    UnloadImage: extern "win64" fn(),
    /// Terminates boot services.
    ExitBootServices: extern "win64" fn(ImageHandle: Handle, MapKey: usize) -> Status,
    /// Returns a monotonically increasing count for the platform.
    GetNextMonotonicCount: extern "win64" fn(),
    /// Stalls the processor.
    Stall: extern "win64" fn(Microseconds: usize) -> Status,
    /// Resets and sets a watchdog timer used during boot services time.
    SetWatchdogTimer: extern "win64" fn(
        Timeout: usize,
        WatchdogCode: u64,
        DataSize: usize,
        WatchdogData: *const u16,
    ) -> Status,
    /// Uses a set of precedence rules to find the best set of drivers to
    /// manage a controller.
    ConnectController: extern "win64" fn(),
    /// Informs a set of drivers to stop managing a controller.
    DisconnectController: extern "win64" fn(),
    /// Adds elements to the list of agents consuming a protocol interface.
    OpenProtocol: extern "win64" fn(),
    /// Removes elements from the list of agents consuming a protocol
    /// interface.
    CloseProtocol: extern "win64" fn(),
    /// Retrieve the list of agents that are currently consuming a
    /// protocol interface.
    OpenProtocolInformation: extern "win64" fn(),
    /// Retrieves the list of protocols installed on a handle. The return
    /// buffer is automatically allocated.
    ProtocolsPerHandle:
        extern "win64" fn(Handle: Handle, ProtocolBuffer: *mut Guid, ProtocolBufferCount: usize)
            -> Status,
    /// Retrieves the list of handles from the handle database that meet
    /// the search criteria. The return buffer is automatically allocated.
    LocateHandleBuffer: extern "win64" fn(
        SearchType: LocateSearchType,
        Protocol: &Guid,
        SearchKey: usize,
        NoHandles: &mut usize,
        Buffer: &mut *mut Handle,
    ),
    /// Finds the first handle in the handle database the supports the requested protocol.
    LocateProtocol:
        extern "win64" fn(Protocol: &Guid, Registration: usize, Interface: &mut usize) -> Status,
    /// Installs one or more protocol interfaces onto a handle.
    InstallMultipleProtocolInterfaces: extern "win64" fn(),
    /// Uninstalls one or more protocol interfaces from a handle.
    UninstallMultipleProtocolInterfaces: extern "win64" fn(),
    /// Computes and returns a 32-bit CRC for a data buffer.
    CalculateCrc32: extern "win64" fn(),
    /// Copies the contents of one buffer to another buffer.
    CopyMem: extern "win64" fn(),
    /// Fills a buffer with a specified value.
    SetMem: extern "win64" fn(),
    /// Creates an event structure as part of an event group.
    CreateEventEx: extern "win64" fn(),
}

impl BootServices {
    /// Stops execution until an event is signaled.
    pub fn wait_for_events<'a>(&self, events: &'a [Event]) -> Result<&'a Event, Error> {
        let mut index = 0;

        (self.WaitForEvent)(events.len(), events.as_ptr(), &mut index)?;

        Ok(events.get(index).expect("UEFI returned the wrong index."))
    }

    /// Stops execution until an event is signaled.
    pub fn wait_for_event(&self, event: &Event) -> Result<(), Error> {
        let mut index = 0;

        (self.WaitForEvent)(1, event, &mut index)?;

        assert!(index == 0, "UEFI returned the wrong index.");

        Ok(())
    }

    /// Allocates pages of a particular type.
    pub fn allocate_pages(&self, memory_type: MemoryType, pages: usize) -> Result<*const u8, Error> {
        let mut address = PhysicalAddress::default();

        (self.AllocatePages)(0, memory_type, pages, &mut address)?;

        Ok(address.0 as *const u8)
    }

    /// Frees allocated pages.
    pub fn free_pages(&self, memory: *const u8, pages: usize) -> Result<(), Error> {
        (self.FreePages)(PhysicalAddress(memory as u64), pages)?;

        Ok(())
    }

    /// Returns the current boot services memory map and memory map key.
    pub fn get_memory_map(&self, memory_type: MemoryType) -> Result<MemoryMap, Error> {
        // The buffer will be allocated on whole pages, that makes it easier to reuse the memory later on.
        // Try one page as a buffer size first.
        let mut memory_map = MemoryMap {
            buffer: self.allocate_pages(memory_type, 1)? as *const MemoryDescriptor,
            alloc_size: 1,
            size: PAGE_SIZE,
            key: 0,
            descriptor_size: 0,
            version: 0,
        };

        loop {
            if (self.GetMemoryMap)(
                &mut memory_map.size,
                memory_map.buffer as *mut MemoryDescriptor,
                &mut memory_map.key,
                &mut memory_map.descriptor_size,
                &mut memory_map.version,
            ) == SUCCESS
            {
                break;
            }

            memory_map.alloc_size = (memory_map.size / PAGE_SIZE) + 1;

            self.free_pages(memory_map.buffer as *const u8, memory_map.alloc_size)?;
            memory_map.buffer =
                self.allocate_pages(memory_type, memory_map.alloc_size)? as *const MemoryDescriptor;

        }

        assert!(
            memory_map.descriptor_size >= size_of::<MemoryDescriptor>(),
            "The size of the memory descriptor is smaller than the standard says."
        );

        Ok(memory_map)
    }

    /// Allocates a pool of a particular type.
    pub fn allocate_pool(&self, memory_type: MemoryType, size: usize) -> Result<*const u8, Error> {
        let mut address = 0;

        (self.AllocatePool)(memory_type, size, &mut address)?;

        Ok(address as *const u8)
    }

    /// Frees allocated pool.
    pub fn free_pool(&self, buffer: *const u8) -> Result<(), Error> {
        (self.FreePool)(buffer as usize)?;

        Ok(())
    }

    /// Terminates boot services if a memory map and its key is already available.
    pub fn exit_boot_services_with_map(
        &self,
        image_handle: Handle,
        map_key: usize,
    ) -> Result<(), Error> {
        (self.ExitBootServices)(image_handle, map_key)?;

        Ok(())
    }

    /// Terminates boot services returning the memory map.
    ///
    /// `memory_type` is the type of memory the caller uses for its data.
    pub fn exit_boot_services(&self, image_handle: Handle) -> Result<MemoryMap, Error> {
        // The data memory type for applications that would call exit_boot_services is assumed to always be `LoaderData`.
        let mut memory_map = self.get_memory_map(MemoryType::LoaderData)?;

        match self.exit_boot_services_with_map(image_handle, memory_map.key) {
            Ok(_) => Ok(()),
            Err(_) => loop {
                // If the call to ExitBootServices failed, the memory map was invalid.
                // We need to try get a new memory map, but we cannot allocate anymore
                // after trying to call ExitBootServices once.
                if (self.GetMemoryMap)(
                    &mut memory_map.size,
                    memory_map.buffer as *mut MemoryDescriptor,
                    &mut memory_map.key,
                    &mut memory_map.descriptor_size,
                    &mut memory_map.version,
                ) != SUCCESS
                {
                    // If the call to GetMemoryMap failed, there is no way to get another buffer.
                    // Therefore we have to abort with an error.
                    break Err(Error::Aborted);
                } else if self
                    .exit_boot_services_with_map(image_handle, memory_map.key)
                    .is_ok()
                {
                    // If the call succeeded, try again.
                    break Ok(());
                }
            },
        }?;

        // Boot services memory can be treated as conventional memory after calling `ExitBootServices`.
        for entry in memory_map.iter_mut() {
            if entry.Type == MemoryType::BootServicesCode
                || entry.Type == MemoryType::BootServicesData
            {
                entry.Type = MemoryType::ConventionalMemory;
            }
        }

        Ok(memory_map)
    }
}
