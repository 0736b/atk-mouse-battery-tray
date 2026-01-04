# ATK Mouse Battery Tray (Windows)

### Setup VID/PID for your ATK Mouse

You can find the Mouse VID/PID using Chrome’s device log

Steps

1. Open Chrome

2. In the address bar, go to:

    ```
    chrome://device-log/
    ```

3. Click 'Refresh'

4. Watch the log for a line like:

    ```
    HID device detected: vendorId=14139, productId=4472, name='Wireless mouse dongle- 8k NANO'
    ```
5. Edit in `hid.rs`
    ```
    // USB:      vendorId=14139, productId=4397, name='ATK A9 PRO'
    // Wireless: vendorId=14139, productId=4472, name='Wireless mouse dongle- 8k NANO'
    pub const VID: u16 = 14139;
    pub const PID_USB: u16 = 4397;
    pub const PID_WIRELESS: u16 = 4472;
    ```

### Build and Run

```
cargo build --release
.\target\release\atk-mouse-battery-tray.exe
```

> Tip: If ATK Hub or other software is actively using the dongle’s HID interface, reading/writing reports may fail. Close it if you run into issues.

