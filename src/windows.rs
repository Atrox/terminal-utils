use std::io;

use windows::core::w;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    GetConsoleMode, GetConsoleScreenBufferInfo, SetConsoleMode, CONSOLE_MODE,
    CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT, ENABLE_EXTENDED_FLAGS, ENABLE_INSERT_MODE,
    ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_QUICK_EDIT_MODE,
    ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_WINDOW_INPUT,
};

use crate::TerminalSize;

const RAW_MODE_MASK: CONSOLE_MODE = CONSOLE_MODE(
    ENABLE_EXTENDED_FLAGS.0
        | ENABLE_INSERT_MODE.0
        | ENABLE_QUICK_EDIT_MODE.0
        | ENABLE_VIRTUAL_TERMINAL_INPUT.0,
);

const NOT_RAW_MODE_MASK: CONSOLE_MODE = CONSOLE_MODE(
    ENABLE_LINE_INPUT.0
        | ENABLE_ECHO_INPUT.0
        | ENABLE_MOUSE_INPUT.0
        | ENABLE_WINDOW_INPUT.0
        | ENABLE_PROCESSED_INPUT.0,
);

#[derive(Debug, Clone, Copy)]
pub struct TerminalState(CONSOLE_MODE);

pub fn size() -> Result<TerminalSize, io::Error> {
    let handle = get_current_out_handle()?;
    let info = get_screen_buffer_info(&handle)?;

    let width = info.srWindow.Right - info.srWindow.Left + 1;
    let height = info.srWindow.Bottom - info.srWindow.Top + 1;
    Ok(TerminalSize {
        width: width as u16,
        height: height as u16,
        pixel_width: 0,
        pixel_height: 0,
    })
}

pub fn is_raw_mode_enabled() -> Result<bool, io::Error> {
    let handle = get_current_in_handle()?;
    let mode = get_console_mode(&handle)?;

    Ok(mode & NOT_RAW_MODE_MASK == CONSOLE_MODE(0) && mode & RAW_MODE_MASK == RAW_MODE_MASK)
}

pub fn enable_raw_mode() -> Result<TerminalState, io::Error> {
    let handle = get_current_in_handle()?;
    let original_mode = get_console_mode(&handle)?;

    let new_mode = original_mode & !NOT_RAW_MODE_MASK | RAW_MODE_MASK;
    set_console_mode(&handle, new_mode)?;

    Ok(TerminalState(original_mode))
}

pub fn restore_mode(original_mode: TerminalState) -> Result<(), io::Error> {
    let handle = get_current_in_handle()?;
    set_console_mode(&handle, original_mode.0)?;

    Ok(())
}

// TODO: check if there is a better way in windows to get notified when the terminal is resized
#[cfg(feature = "tokio")]
pub fn spawn_on_resize_task(
    tx: tokio::sync::watch::Sender<TerminalSize>,
) -> Result<tokio::task::JoinHandle<()>, io::Error> {
    let task = tokio::spawn(async move {
        loop {
            if tx.is_closed() {
                break;
            }

            if let Ok(size) = size() {
                tx.send_if_modified(|current_size| {
                    if current_size != &size {
                        *current_size = size;
                        true
                    } else {
                        false
                    }
                });
            };

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
    Ok(task)
}

fn get_current_in_handle() -> Result<HANDLE, io::Error> {
    get_handle(w!("CONIN$"))
}

fn get_current_out_handle() -> Result<HANDLE, io::Error> {
    get_handle(w!("CONOUT$"))
}

fn get_handle<P0>(name: P0) -> Result<HANDLE, io::Error>
where
    P0: windows::core::IntoParam<::windows::core::PCWSTR>,
{
    let handle = unsafe {
        CreateFileW(
            name,
            (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };

    Ok(handle)
}

fn get_console_mode(handle: &HANDLE) -> Result<CONSOLE_MODE, io::Error> {
    let mut mode: CONSOLE_MODE = CONSOLE_MODE::default();
    unsafe { GetConsoleMode(*handle, &mut mode)? }

    Ok(mode)
}

fn set_console_mode(handle: &HANDLE, mode: CONSOLE_MODE) -> Result<(), io::Error> {
    unsafe { SetConsoleMode(*handle, mode)? }

    Ok(())
}

fn get_screen_buffer_info(handle: &HANDLE) -> Result<CONSOLE_SCREEN_BUFFER_INFO, io::Error> {
    let mut info: CONSOLE_SCREEN_BUFFER_INFO = CONSOLE_SCREEN_BUFFER_INFO::default();
    unsafe { GetConsoleScreenBufferInfo(*handle, &mut info)? }

    Ok(info)
}
