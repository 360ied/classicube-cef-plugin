mod cef;
mod error;
mod helpers;

use classicube_sys::*;
use std::{cell::Cell, ffi::CString, os::raw::c_int, ptr};

extern "C" fn init() {
    color_backtrace::install_with_settings(
        color_backtrace::Settings::new().verbosity(color_backtrace::Verbosity::Full),
    );

    {
        let append_app_name = CString::new(format!(" +cef{}", env!("CARGO_PKG_VERSION"))).unwrap();

        let c_str = append_app_name.as_ptr();

        unsafe {
            String_AppendConst(&mut Server.AppName, c_str);
        }
    }

    cef::initialize();
}

extern "C" fn free() {
    println!("Free");

    cef::shutdown();
}

thread_local!(
    static CONTEXT_LOADED: Cell<bool> = Cell::new(false);
);

extern "C" fn on_new_map_loaded() {
    println!("OnNewMapLoaded");

    CONTEXT_LOADED.with(|cell| {
        if !cell.get() {
            cell.set(true);
            cef::on_first_context_created();
        }
    });
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
    // Called when the game is being loaded.
    Init: Some(init),
    // Called when the component is being freed. (e.g. due to game being closed)
    Free: Some(free),
    // Called to reset the component's state. (e.g. reconnecting to server)
    Reset: None,
    // Called to update the component's state when the user begins loading a new map.
    OnNewMap: None,
    // Called to update the component's state when the user has finished loading a new map.
    OnNewMapLoaded: Some(on_new_map_loaded),
    // Next component in linked list of components.
    next: ptr::null_mut(),
};
