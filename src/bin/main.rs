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
use libc::{
    c_char,
    c_void,
};

use disk_notify::DiskInfo;

static mut CHANNEL: MaybeUninit<(Sender<Option<DiskInfo>>, Receiver<Option<DiskInfo>>)> = MaybeUninit::uninit();
static CHANNEL_INIT: Once = Once::new();

#[inline]
fn get_channel() -> &'static (Sender<Option<DiskInfo>>, Receiver<Option<DiskInfo>>) {
    CHANNEL_INIT.call_once(|| unsafe {
        ptr::write(CHANNEL.as_mut_ptr(), channel::unbounded());
    });

    unsafe {
        &*CHANNEL.as_ptr()
    }
}

extern "C" fn disk_appeared_callback(disk: da::disk::DADiskRef, _context: da::types::UnsafeMutableRawPointer) {
    let (tx, _rx) = get_channel();
    let info = DiskInfo::from_disk_ref(disk).ok();
    tx.send(info).unwrap();
}

pub fn main() {

    let (session_send, session_recv) = channel::bounded::<Option<&da::session::_DASession>>(1);

    thread::spawn(|| {
        let session_send = session_send;

        unsafe {
            let session = da::session::DASessionCreate(da::enums::kCFAllocatorDefault);

            da::arbitration::DARegisterDiskAppearedCallback(
                session,
                da::enums::kDADiskDescriptionMatchVolumeMountable,
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
            println!("{:#?}", data);
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
