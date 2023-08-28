use std::fmt::Debug;
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::{io, mem};

use crate::TerminalSize;

#[derive(Clone, Copy)]
pub struct TerminalState(libc::termios);

impl Debug for TerminalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TerminalState")
            .field(&self.0.c_iflag)
            .field(&self.0.c_oflag)
            .field(&self.0.c_cflag)
            .field(&self.0.c_lflag)
            .field(&self.0.c_cc)
            .field(&self.0.c_ispeed)
            .field(&self.0.c_ospeed)
            .finish()
    }
}

pub fn size() -> Result<TerminalSize, io::Error> {
    let tty = get_tty()?;
    let fd = tty.as_raw_fd();

    let info = get_winsize(fd)?;

    Ok(TerminalSize {
        width: info.ws_col,
        height: info.ws_row,

        pixel_width: info.ws_xpixel,
        pixel_height: info.ws_ypixel,
    })
}

pub fn is_raw_mode_enabled() -> Result<bool, io::Error> {
    let tty = get_tty()?;
    let fd = tty.as_raw_fd();

    let termios = get_terminal_attr(fd)?;
    Ok((termios.c_lflag & libc::ICANON) == 0)
}

pub fn enable_raw_mode() -> Result<TerminalState, io::Error> {
    let tty = get_tty()?;
    let fd = tty.as_raw_fd();

    let mut termios = get_terminal_attr(fd)?;
    let original_termios = termios;

    unsafe { libc::cfmakeraw(&mut termios) };
    set_terminal_attr(fd, &termios)?;

    Ok(TerminalState(original_termios))
}

pub fn restore_mode(original_termios: TerminalState) -> Result<(), io::Error> {
    let tty = get_tty()?;
    let fd = tty.as_raw_fd();

    set_terminal_attr(fd, &original_termios.0)?;

    Ok(())
}

#[cfg(feature = "tokio")]
pub fn spawn_on_resize_task(
    tx: tokio::sync::watch::Sender<TerminalSize>,
) -> Result<tokio::task::JoinHandle<()>, io::Error> {
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::window_change())?;

    let task = tokio::spawn(async move {
        loop {
            signal.recv().await;

            if let Ok(size) = size() {
                tx.send_replace(size);
            }
        }
    });
    Ok(task)
}

fn get_tty() -> Result<File, io::Error> {
    File::open("/dev/tty")
}

fn get_winsize(fd: RawFd) -> Result<libc::winsize, io::Error> {
    let mut info: libc::winsize = unsafe { mem::zeroed() };
    wrap_error(unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut info) })?;

    Ok(info)
}

fn get_terminal_attr(fd: RawFd) -> Result<libc::termios, io::Error> {
    let mut termios: libc::termios = unsafe { mem::zeroed() };
    wrap_error(unsafe { libc::tcgetattr(fd, &mut termios) })?;

    Ok(termios)
}

fn set_terminal_attr(fd: RawFd, termios: &libc::termios) -> Result<(), io::Error> {
    wrap_error(unsafe { libc::tcsetattr(fd, libc::TCSANOW, termios) })?;

    Ok(())
}

fn wrap_error(result: libc::c_int) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
