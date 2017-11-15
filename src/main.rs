extern crate libc;
extern crate widestring;
extern crate winapi;
extern crate named_pipe;

use libc::size_t;
use std::ptr;
use widestring::WideCString;
use named_pipe::PipeClient;
use std::io::Read;
use std::io::Write;
use std::io::copy;
use std::thread;

#[link(name = "winpty")]
extern {
    fn winpty_config_new(agentFlags: u64, err: *mut size_t) -> size_t;
    fn winpty_config_set_initial_size(cfg: size_t, cols: u32, rows: u32);
    fn winpty_open(cfg: size_t, err: *mut size_t) -> size_t;
    fn winpty_spawn_config_new(spawnFlags: u64, appname: *const u16, cmdline: *const u16, cwd: *const u16, env: *const u16, err: *mut size_t) -> size_t;
    fn winpty_conin_name(wp: size_t) -> *const u16;
    fn winpty_conout_name(wp: size_t) -> *const u16;
    fn winpty_conerr_name(wp: size_t) -> *const u16;
    fn winpty_spawn(wp: size_t, cfg: size_t, process_handle: *mut size_t, thread_handle: *mut size_t, create_process_error: *mut size_t, err: *mut size_t) -> bool;
    fn winpty_error_code(err: size_t) -> u16;
    fn winpty_error_msg(err: size_t) -> *const u16;
}

const WINPTY_FLAG_COLOR_ESCAPES: u64 = 0x4;
const WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN: u64 = 1;

fn main() {
    unsafe {
        let cfg_error = &mut 0usize;
        let cfg_ptr = winpty_config_new(WINPTY_FLAG_COLOR_ESCAPES, cfg_error);
        // println!("cfg = {}", cfg_ptr);
        // println!("Cfg Message: {}", WideCString::from_ptr_str(winpty_error_msg(*cfg_error)).to_string().unwrap());

        winpty_config_set_initial_size(cfg_ptr, 80, 32);

        let open_error = &mut 0usize;
        let handle = winpty_open(cfg_ptr, open_error);
        // println!("console handle = {}", handle);
        // println!("Open Message: {}", WideCString::from_ptr_str(winpty_error_msg(*open_error)).to_string().unwrap());

        let exe = WideCString::from_str("c:/Windows/System32/WindowsPowershell/v1.0/powershell.exe").unwrap();
        let args = WideCString::from_str(" -NoExit -c \"remove-module psreadline\"").unwrap();
        let cwd = WideCString::from_str("c:/").unwrap();
        let spawn_config_error = &mut 0usize;
        let spawn_config = winpty_spawn_config_new(WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN, exe.as_ptr(), args.as_ptr(), cwd.as_ptr(), ptr::null_mut(), spawn_config_error);
        // println!("Spawn CFG Message: {}", WideCString::from_ptr_str(winpty_error_msg(*spawn_config_error)).to_string().unwrap());

        let stdin_name = WideCString::from_ptr_str(winpty_conin_name(handle)).to_os_string();
        let stdout_name = WideCString::from_ptr_str(winpty_conout_name(handle)).to_os_string();
        // println!("in pipe = {}", stdin_name.to_str().unwrap());
        // println!("out pipe = {}", stdout_name.to_str().unwrap());

        let mut in_pipe = PipeClient::connect(&stdin_name).expect("In pipe failed");
        let mut out_pipe = PipeClient::connect(&stdout_name).expect("Out pipe failed");

        let process = &mut 0usize;
        let thread = &mut 0usize;
        let proc_error = &mut 0usize;
        let spawn_error = &mut 0usize;
        winpty_spawn(handle, spawn_config, process, thread, proc_error, spawn_error);
        // println!("Proc Message: {}", WideCString::from_ptr_str(winpty_error_msg(*proc_error)).to_string().unwrap());
        // println!("Spawn Message: {}", WideCString::from_ptr_str(winpty_error_msg(*spawn_error)).to_string().unwrap());

        thread::spawn(move || {
            copy(&mut out_pipe, &mut std::io::stdout()).expect("Failed to pipe output");
        });
        copy(&mut std::io::stdin(), &mut in_pipe).expect("Failed to pipe input");

    }
}
