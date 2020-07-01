use core_foundation as cf;
use disk_arbitration as da;

use crossbeam_channel::{
    self as channel,
    Receiver,
    Sender,
};
// use crossbeam_utils::thread;
use std::{
    ffi::CString,
    mem::MaybeUninit,
    ptr,
    sync::Once,
    thread,
};
use libc::c_void;

static mut CHANNEL: MaybeUninit<(Sender<Option<CString>>, Receiver<Option<CString>>)> = MaybeUninit::uninit();
static CHANNEL_INIT: Once = Once::new();

#[inline]
fn get_channel() -> &'static (Sender<Option<CString>>, Receiver<Option<CString>>) {
    CHANNEL_INIT.call_once(|| unsafe {
        ptr::write(CHANNEL.as_mut_ptr(), channel::unbounded());
    });

    unsafe {
        &*CHANNEL.as_ptr()
    }
}

extern "C" fn disk_appeared_callback(disk: da::disk::DADiskRef, _context: da::types::UnsafeMutableRawPointer) {
    unsafe {
        let name = da::disk::DADiskGetBSDName(disk);
        let (tx, _rx) = get_channel();

        if !name.is_null() {
            // turn the pointer into a CStr
            let cstr = std::ffi::CStr::from_ptr(name);
            // Copy the data into an owned CString
            let name_string: CString = cstr.into();
            tx.send(Some(name_string)).unwrap();
        } else {
            tx.send(None).unwrap();
        }
    }
}

pub fn main() {

    let (session_send, session_recv) = channel::bounded::<Option<&da::session::_DASession>>(1);

    thread::spawn(|| {
        let session_send = session_send;

        unsafe {
            let session = da::session::DASessionCreate(da::enums::kCFAllocatorDefault);

            da::arbitration::DARegisterDiskAppearedCallback(
                session,
                std::ptr::null(),
                Some(disk_appeared_callback),
                std::ptr::null_mut(),
            );

            da::session::DASessionScheduleWithRunLoop(
                session,
                cf::runloop::CFRunLoopGetCurrent(),
                cf::runloop::kCFRunLoopDefaultMode,
            );

            cf::runloop::CFRunLoopRun();

            session_send.send(session.as_ref()).expect("failed to send session!");
        };
    });

    let (_tx, rx) = get_channel();

    for maybe_string in rx.iter() {
        if let Some(data) = maybe_string {
            println!("{:?}", data);
        }
    };

    unsafe {
        let session: da::session::DASessionRef = session_recv.try_recv().unwrap().unwrap();

        da::session::DASessionUnscheduleFromRunLoop(
            session,
            cf::runloop::CFRunLoopGetCurrent(),
            cf::runloop::kCFRunLoopDefaultMode,
        );

        cf::base::CFRelease(session as *const c_void);
    }

    print!("Hello World!");
}
