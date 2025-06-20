use std::{
    ffi::{CStr, c_void},
    ops::Deref,
    ptr::null_mut,
};

use crate::{ThreadSafePointer, sys};

#[derive(Debug)]
pub struct Request<'a> {
    pub url: &'a str,
    pub method: &'a str,
    pub referrer: &'a str,
}

impl<'a> Request<'a> {
    fn from_raw_ptr(request: *mut sys::ResourceRequest) -> Option<Self> {
        let request = unsafe { &*request };

        Some(Self {
            url: unsafe { CStr::from_ptr(request.url).to_str().ok()? },
            method: unsafe { CStr::from_ptr(request.method).to_str().ok()? },
            referrer: unsafe { CStr::from_ptr(request.referrer).to_str().ok()? },
        })
    }
}

pub trait ResourceHandler: Send + Sync {
    fn open(&self) -> bool;

    fn get_response(&self, response: &mut sys::ResourceResponse);

    fn skip(&self, size: usize, skip_bytes: &mut usize) -> bool;

    fn read(&self, buffer: &mut [u8], read_bytes: &mut usize) -> bool;

    fn cancel(&self);
}

impl ResourceHandler for Box<dyn ResourceHandler> {
    fn open(&self) -> bool {
        self.as_ref().open()
    }

    fn get_response(&self, response: &mut sys::ResourceResponse) {
        self.as_ref().get_response(response)
    }

    fn skip(&self, size: usize, skip_bytes: &mut usize) -> bool {
        self.as_ref().skip(size, skip_bytes)
    }

    fn read(&self, buffer: &mut [u8], read_bytes: &mut usize) -> bool {
        self.as_ref().read(buffer, read_bytes)
    }

    fn cancel(&self) {
        self.as_ref().cancel()
    }
}

pub trait RequestHandler: Send + Sync {
    fn on_request(&self, request: &Request) -> Option<Box<dyn ResourceHandler>>;
}

pub struct RequestFilter {
    raw: ThreadSafePointer<Box<dyn RequestHandler>>,
    pub(crate) raw_handler: sys::ResourceRequestHandler,
}

impl RequestFilter {
    pub fn new<T>(handler: T) -> Self
    where
        T: RequestHandler + 'static,
    {
        let raw: *mut Box<dyn RequestHandler> = Box::into_raw(Box::new(Box::new(handler)));
        let raw_handler = sys::ResourceRequestHandler {
            create_resource_handler: Some(on_create_resource_handler),
            destroy_resource_handler: Some(on_destroy_resource_handler),
            context: raw as _,
        };

        Self {
            raw: ThreadSafePointer(raw),
            raw_handler,
        }
    }
}

impl Deref for RequestFilter {
    type Target = sys::ResourceRequestHandler;

    fn deref(&self) -> &Self::Target {
        &self.raw_handler
    }
}

impl Drop for RequestFilter {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.raw.as_ptr()) });
    }
}

extern "C" fn on_create_resource_handler(
    request: *mut sys::ResourceRequest,
    context: *mut c_void,
) -> *mut sys::ResourceHandler {
    if request.is_null() {
        return null_mut();
    }

    if let Some(request) = Request::from_raw_ptr(request) {
        if let Some(handler) =
            unsafe { &*(context as *mut Box<dyn RequestHandler>) }.on_request(&request)
        {
            return Box::into_raw(Box::new(sys::ResourceHandler {
                open: Some(on_open),
                skip: Some(on_skip),
                read: Some(on_read),
                cancel: Some(on_cancel),
                destroy: Some(on_destroy),
                get_response: Some(on_get_response),
                context: Box::into_raw(Box::new(handler)) as _,
            })) as _;
        }
    }

    null_mut()
}

extern "C" fn on_destroy_resource_handler(handler: *mut sys::ResourceHandler) {
    drop(unsafe { Box::from_raw(handler) });
}

extern "C" fn on_open(context: *mut c_void) -> bool {
    unsafe { &*(context as *mut Box<dyn ResourceHandler>) }.open()
}

extern "C" fn on_get_response(response: *mut sys::ResourceResponse, context: *mut c_void) {
    unsafe { &*(context as *mut Box<dyn ResourceHandler>) }.get_response(unsafe { &mut *response });
}

extern "C" fn on_skip(size: usize, skip_bytes: *mut usize, context: *mut c_void) -> bool {
    unsafe { &*(context as *mut Box<dyn ResourceHandler>) }.skip(size, unsafe { &mut *skip_bytes })
}

extern "C" fn on_read(
    buffer: *mut c_void,
    size: usize,
    read_bytes: *mut usize,
    context: *mut c_void,
) -> bool {
    unsafe { &*(context as *mut Box<dyn ResourceHandler>) }.read(
        unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, size) },
        unsafe { &mut *read_bytes },
    )
}

extern "C" fn on_cancel(context: *mut c_void) {
    unsafe { &*(context as *mut Box<dyn ResourceHandler>) }.cancel();
}

extern "C" fn on_destroy(context: *mut c_void) {
    drop(unsafe { Box::from_raw(context as *mut Box<dyn ResourceHandler>) });
}
