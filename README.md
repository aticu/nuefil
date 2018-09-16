# nuefil - New UEFI Library
nuefil is yet another Rust [UEFI](http://www.uefi.org/sites/default/files/resources/UEFI%20Spec%202_7_A%20Sept%206.pdf) libraray.

It is based on the [UEFI libaray of Redox](https://gitlab.redox-os.org/redox-os/uefi).

## Using nuefil
To use nuefil just write the following in your Cargo.toml and use the types where the FFI boundaries require them.

```toml
nuefil = { git = "https://github.com/aticu/nuefil" }
```

Then you can use it in the following way:

```rust
use core::fmt::Write;
use nuefil::{
    Handle,
    status::{Status, SUCCESS},
    system::SystemTable,
    text::{BackgroundColor, Color, ForegroundColor},
};

// The entry point for UEFI.
pub extern "C" fn efi_main(_image_handle: Handle, system_table: &'static SystemTable) -> Status {
    (&*system_table.ConsoleOut).write_fmt(format_args!("Testing the {} library.\r\n", "nuefil")).unwrap();

    system_table.ConsoleOut.set_attribute(Color::new(ForegroundColor::Red, BackgroundColor::Blue)).unwrap();

    (&*system_table.ConsoleOut).write_fmt(format_args!("This line should be red with a blue background.")).unwrap();

    // Wait for input, so the output can be read
    system_table.ConsoleIn.read_key_stroke(system_table).unwrap();

    SUCCESS
}
```

If you're unfamiliar working Rust in such an environment and get stuck on the errors, try reading [this](https://os.phil-opp.com/) excellent block. It talks about Rust without an operating system in general. For UEFI specifics, you can check out [this](https://github.com/phil-opp/blog_os/issues/349) thread.