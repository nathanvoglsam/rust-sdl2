extern crate raw_window_handle;

use self::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use crate::{sys::SDL_Window, video::Window};
use std::ptr::NonNull;

unsafe impl HasRawWindowHandle for Window {
    #[doc(alias = "SDL_GetVersion")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        use self::SDL_SYSWM_TYPE::*;

        let mut wm_info: SDL_SysWMinfo = unsafe { std::mem::zeroed() };

        // Make certain to retrieve version before querying `SDL_GetWindowWMInfo`
        // as that gives an error on certain systems
        unsafe {
            sys::SDL_GetVersion(&mut wm_info.version);
            if SDL_GetWindowWMInfo(self.raw(), &mut wm_info) == SDL_bool::SDL_FALSE {
                panic!("Couldn't get SDL window info: {}", crate::get_error());
            }
        }

        match wm_info.subsystem {
            #[cfg(target_os = "windows")]
            SDL_SYSWM_WINDOWS => {
                use self::raw_window_handle::Win32;
                let info = unsafe { wm_info.info.win.window };
                let info = info as *mut core::ffi::c_void;
                let info = NonNull::new(info);
                let mut handle = WindowsHandle::empty();
                handle.hwnd = info;
                RawWindowHandle::Win32(handle)
            }
            #[cfg(target_os = "windows")]
            SDL_SYSWM_WINRT => {
                use self::raw_window_handle::WinRTHandle;
                let core_window = unsafe { wm_info.info.uwp.core_window };
                let core_window = NonNull::new(core_window);
                let mut handle = WinRTHandle::empty();
                handle.core_window = core_window;
                RawWindowHandle::WinRT(handle)
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            SDL_SYSWM_WAYLAND => {
                use self::raw_window_handle::unix::WaylandHandle;
                let mut info = WaylandHandle::empty();
                info.surface = unsafe { wm_info.info.wl }.surface as *mut core::ffi::c_void;
                info.display = unsafe { wm_info.info.wl }.display as *mut core::ffi::c_void;
                RawWindowHandle::Wayland(info)
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            SDL_SYSWM_X11 => {
                use self::raw_window_handle::unix::XlibHandle;
                let mut info = XlibHandle::empty();
                info.window = unsafe { wm_info.info.x11 }.window;
                info.display = unsafe { wm_info.info.x11 }.display as *mut core::ffi::c_void;
                RawWindowHandle::Xlib(info)
            }
            #[cfg(target_os = "macos")]
            SDL_SYSWM_COCOA => {
                use self::raw_window_handle::macos::MacOSHandle;
                let mut info = MacOSHandle::empty();
                let ns_window = unsafe { wm_info.info.cocoa }.window as *mut core::ffi::c_void;
                info.ns_window = NonNull::new(ns_window);
                info.ns_view = None; // consumer of RawWindowHandle should determine this
                RawWindowHandle::MacOS(info)
            }
            #[cfg(any(target_os = "ios"))]
            SDL_SYSWM_UIKIT => {
                use self::raw_window_handle::ios::IOSHandle;
                let mut info = IOSHandle::empty();
                let ui_window = unsafe { wm_info.info.uikit }.window as *mut core::ffi::c_void;
                info.ui_window = NonNull::new(ui_window);
                info.ui_view = None;
                RawWindowHandle::IOS(IOSHandle {
                    ui_window: unsafe { wm_info.info.uikit }.window as *mut libc::c_void,
                    ui_view: 0 as *mut libc::c_void, // consumer of RawWindowHandle should determine this
                    ..IOSHandle::empty()
                })
            }
            SDL_SYSWM_ANDROID => {
                let window_system = match wm_info.subsystem {
                    SDL_SYSWM_ANDROID => "Android",
                    _ => unreachable!(),
                };
                panic!(
                    "raw-window-handle support for {} not yet implemented",
                    window_system
                );
            }
            x => {
                let window_system = match x {
                    SDL_SYSWM_DIRECTFB => "DirectFB",
                    SDL_SYSWM_MIR => "Mir",
                    SDL_SYSWM_VIVANTE => "Vivante",
                    _ => unreachable!(),
                };
                panic!("{} window system is not supported, please file issue with raw-window-handle: https://github.com/rust-windowing/raw-window-handle/issues/new", window_system);
            }
        }
    }
}

extern "C" {
    fn SDL_GetWindowWMInfo(window: *mut SDL_Window, info: *mut SDL_SysWMinfo) -> SDL_bool;
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types, dead_code)]
pub enum SDL_bool {
    SDL_FALSE = 0,
    SDL_TRUE = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types, dead_code)]
pub enum SDL_SYSWM_TYPE {
    SDL_SYSWM_UNKNOWN = 0,
    SDL_SYSWM_WINDOWS = 1,
    SDL_SYSWM_X11 = 2,
    SDL_SYSWM_DIRECTFB = 3,
    SDL_SYSWM_COCOA = 4,
    SDL_SYSWM_UIKIT = 5,
    SDL_SYSWM_WAYLAND = 6,
    SDL_SYSWM_MIR = 7,
    SDL_SYSWM_WINRT = 8,
    SDL_SYSWM_ANDROID = 9,
    SDL_SYSWM_VIVANTE = 10,
    SDL_SYSWM_OS2 = 11,
}

impl Default for SDL_SYSWM_TYPE {
    fn default() -> Self {
        SDL_SYSWM_TYPE::SDL_SYSWM_UNKNOWN
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct SDL_SysWMinfo {
    pub version: sys::SDL_version,
    pub subsystem: SDL_SYSWM_TYPE,

    #[cfg(target_os = "windows")]
    pub info: windows::WindowsSysWMinfo,

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub info: linux::LinuxSysWMinfo,

    #[cfg(target_os = "macos")]
    pub info: macos::MacOSSysWMinfo,

    #[cfg(target_os = "ios")]
    pub info: ios::IOSSysWMinfo,
}

#[cfg(target_os = "windows")]
pub mod windows {
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union WindowsSysWMinfo {
        pub win: Win32Handles,
        pub uwp: WinRTHandles,
        pub dummy: [u8; 64usize],
        _bindgen_union_align: [u64; 8usize],
    }

    impl Default for WindowsSysWMinfo {
        fn default() -> Self {
            WindowsSysWMinfo {
                win: Win32Handles::default(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Win32Handles {
        pub window: *mut HWND,
        pub hdc: *mut HDC,
        pub hinstance: *mut HINSTANCE,
    }

    impl Default for Win32Handles {
        fn default() -> Self {
            Win32Handles {
                window: 0 as *mut HWND,
                hdc: 0 as *mut HDC,
                hinstance: 0 as *mut HINSTANCE,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct WinRTHandles {
        pub core_window: *mut core::ffi::c_void,
    }

    impl Default for WinRTHandles {
        fn default() -> Self {
            WinRTHandles {
                core_window: core::ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct HWND {
        pub unused: libc::c_int,
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct HDC {
        pub unused: libc::c_int,
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct HINSTANCE {
        pub unused: libc::c_int,
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
pub mod linux {
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union LinuxSysWMinfo {
        pub x11: X11Info,
        pub wl: WaylandInfo,
        pub dummy: [u8; 64usize],
        _bindgen_union_align: [u32; 16usize],
    }

    impl Default for LinuxSysWMinfo {
        fn default() -> Self {
            LinuxSysWMinfo {
                wl: WaylandInfo::default(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct X11Info {
        pub display: *mut Display,
        pub window: Window,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct WaylandInfo {
        pub display: *mut WLDisplay,
        pub surface: *mut WLSurface,
        pub shell_surface: *mut WLShellSurface,
    }

    impl Default for WaylandInfo {
        fn default() -> Self {
            WaylandInfo {
                display: 0 as *mut WLDisplay,
                surface: 0 as *mut WLSurface,
                shell_surface: 0 as *mut WLShellSurface,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct WLDisplay {
        pub _address: u8,
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct WLSurface {
        pub _address: u8,
    }

    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct WLShellSurface {
        pub _address: u8,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Display {
        _unused: [u8; 0],
    }

    pub type Window = libc::c_ulong;
}

#[cfg(target_os = "macos")]
pub mod macos {
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union MacOSSysWMinfo {
        pub cocoa: CocoaInfo,
        pub dummy: [u8; 64usize],
        _bindgen_union_align: [u64; 8usize],
    }

    impl Default for MacOSSysWMinfo {
        fn default() -> Self {
            MacOSSysWMinfo {
                cocoa: CocoaInfo::default(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct CocoaInfo {
        pub window: *mut NSWindow,
    }

    impl Default for CocoaInfo {
        fn default() -> Self {
            CocoaInfo {
                window: 0 as *mut NSWindow,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct NSWindow {
        _unused: [u8; 0],
    }
}

#[cfg(target_os = "ios")]
pub mod ios {
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union IOSSysWMinfo {
        pub uikit: UiKitInfo,
        pub dummy: [u8; 64usize],
        _bindgen_union_align: [u64; 8usize],
    }

    impl Default for IOSSysWMinfo {
        fn default() -> Self {
            IOSSysWMinfo {
                uikit: UiKitInfo::default(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct UiKitInfo {
        pub window: *mut UIWindow,
    }

    impl Default for UiKitInfo {
        fn default() -> Self {
            UiKitInfo {
                window: 0 as *mut UIWindow,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct UIWindow {
        _unused: [u8; 0],
    }
}
