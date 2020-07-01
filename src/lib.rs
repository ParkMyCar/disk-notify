use core_foundation::{
    string::kCFStringEncodingUTF8,
    self as cf,
};
use disk_arbitration::{
    enums::{
        kDADiskDescriptionDevicePathKey,
        kDADiskDescriptionMediaNameKey,
        kDADiskDescriptionVolumeNameKey,
        kDADiskDescriptionVolumePathKey,
    },
    self as da,
};
use libc::c_void;
use std::ffi::{
    CStr,
    CString,
};

#[derive(Debug, Default)]
pub struct DiskInfo {
    pub bsd_name: Option<CString>,
    pub device_path: Option<CString>,
    pub media_name: Option<CString>,
    pub volume_name: Option<CString>,
    pub volume_path: Option<CString>,
}

impl DiskInfo {
    pub fn from_disk_ref(disk: da::disk::DADiskRef) -> Result<DiskInfo, ()> {
        if disk.is_null() {
            return Err(());
        }

        let mut info = DiskInfo::default();

        let name_ptr = unsafe { da::disk::DADiskGetBSDName(disk) };
        let desc_ptr = unsafe { da::disk::DADiskCopyDescription(disk) };

        if !name_ptr.is_null() {
            let cstr = unsafe { CStr::from_ptr(name_ptr) };
            let bytes = cstr.to_bytes().to_vec();
            let name_string = unsafe { CString::from_vec_unchecked(bytes) };

            info.bsd_name = Some(name_string);
        };

        if !desc_ptr.is_null() {
            // Media Name
            let mn_is_present = unsafe { cf::dictionary::CFDictionaryContainsKey(desc_ptr, kDADiskDescriptionMediaNameKey as *const c_void) };
            if mn_is_present == 1 {
                let mn_ptr = unsafe { da::utils::CFDictionaryGetValue(desc_ptr, kDADiskDescriptionMediaNameKey) };
                if !mn_ptr.is_null() {
                    let char_ptr = unsafe { cf::string::CFStringGetCStringPtr(mn_ptr as cf::string::CFStringRef, kCFStringEncodingUTF8) };
                    if !char_ptr.is_null() {
                        let cstr = unsafe { CStr::from_ptr(char_ptr) };
                        let bytes = cstr.to_bytes().to_vec();
                        
                        let media_name_string = unsafe { CString::from_vec_unchecked(bytes) };
                        info.media_name = Some(media_name_string);
                    }
                }
            }

            // Volume Path
            let vpath_is_present = unsafe { cf::dictionary::CFDictionaryContainsKey(desc_ptr, kDADiskDescriptionVolumePathKey as *const c_void) };
            if vpath_is_present == 1 {
                let vpath_url_ptr = unsafe { da::utils::CFDictionaryGetValue(desc_ptr, kDADiskDescriptionVolumePathKey) };
                if !vpath_url_ptr.is_null() {
                    let vpath_char_ptr = unsafe { cf::url::CFURLCopyPath(vpath_url_ptr as cf::url::CFURLRef) };
                    if !vpath_char_ptr.is_null() {
                        let char_ptr = unsafe { cf::string::CFStringGetCStringPtr(vpath_char_ptr, kCFStringEncodingUTF8) };
                        if !char_ptr.is_null() {
                            let cstr = unsafe { CStr::from_ptr(char_ptr) };
                            let bytes = cstr.to_bytes().to_vec();
                            
                            let media_name_string = unsafe { CString::from_vec_unchecked(bytes) };
                            info.media_name = Some(media_name_string);
                        }
                    }
                }
            }

            // Volume Name
            let vn_is_present = unsafe { cf::dictionary::CFDictionaryContainsKey(desc_ptr, kDADiskDescriptionVolumeNameKey as *const c_void) };
            if vn_is_present == 1 {
                let vn_ptr = unsafe { da::utils::CFDictionaryGetValue(desc_ptr, kDADiskDescriptionVolumeNameKey) };
                if !vn_ptr.is_null() {
                    let char_ptr = unsafe { cf::string::CFStringGetCStringPtr(vn_ptr as cf::string::CFStringRef, kCFStringEncodingUTF8) };
                    if !char_ptr.is_null() {
                        let cstr = unsafe { CStr::from_ptr(char_ptr) };
                        let bytes = cstr.to_bytes().to_vec();
                        
                        let volume_name_string = unsafe { CString::from_vec_unchecked(bytes) };
                        info.volume_name = Some(volume_name_string);
                    }
                }
            }

            // Device Path
            let dpath_is_present = unsafe { cf::dictionary::CFDictionaryContainsKey(desc_ptr, kDADiskDescriptionDevicePathKey as *const c_void) };
            if dpath_is_present == 1 {
                let dpath_ptr = unsafe { da::utils::CFDictionaryGetValue(desc_ptr, kDADiskDescriptionDevicePathKey) };
                if !dpath_ptr.is_null() {
                    let char_ptr = unsafe { cf::string::CFStringGetCStringPtr(dpath_ptr as cf::string::CFStringRef, kCFStringEncodingUTF8) };
                    if !char_ptr.is_null() {
                        let cstr = unsafe { CStr::from_ptr(char_ptr) };
                        let bytes = cstr.to_bytes().to_vec();
                        
                        let device_path_string = unsafe { CString::from_vec_unchecked(bytes) };
                        info.device_path = Some(device_path_string);
                    }
                }
            }
        };

        Ok(info)
    }
}