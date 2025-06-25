use std::{
    ffi::{CStr, CString, c_void},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    ptr::null_mut,
    sync::Arc,
};

use url::Url;

use crate::{sys, utils::ThreadSafePointer};

struct LocalDiskRequestHandler {
    file: Option<File>,
    path: PathBuf,
}

impl LocalDiskRequestHandler {
    fn new(path: PathBuf) -> Self {
        Self { file: None, path }
    }
}

impl RequestHandler for LocalDiskRequestHandler {
    fn open(&mut self) -> bool {
        if let Ok(file) = File::open(&self.path) {
            self.file.replace(file);

            true
        } else {
            false
        }
    }

    fn get_response(&mut self) -> Option<Response> {
        Some(Response {
            status_code: 200,
            mime_type: get_mime_type(&self.path.as_path())?,
            content_length: self.file.as_ref()?.metadata().ok()?.len(),
        })
    }

    fn skip(&mut self, size: usize) -> Option<usize> {
        Some(
            self.file
                .as_mut()?
                .seek(SeekFrom::Start(size as u64))
                .ok()? as usize,
        )
    }

    fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        Some(self.file.as_mut()?.read(buffer).ok()?)
    }

    fn cancel(&mut self) {
        drop(self.file.take());
    }
}

/// This request handler is used to quickly map to the local file system.
///
/// Used to quickly map to the local file system when you need to quickly map
/// static resource files.
///
/// ## Example
///
/// If your scheme is `webview://localhost`, and the local directory you map is
/// `/assets`, then the redirect examples are as follows:
///
/// ```text
/// webview://localhost -> /assets/index.html
/// webview://localhost/index.html -> /assets/index.html
/// webview://localhost/index.css -> /assets/index.css
/// webview://localhost/images/a.jpg -> /assets/images/a.jpg
/// ```
///
/// Besides using it for custom schemes, you can also use it for `WebView`
/// request interception in the `request_handler_factory` of `WebView`.
///
/// Because this request handler will always remove the request protocol header
/// and host, it can be used in different scenarios. For example,
/// `http://localhost/hello/hello.html` actually uses `hello/hello.html`
/// internally.
pub struct RequestHandlerWithLocalDisk {
    root_dir: PathBuf,
}

impl RequestHandlerWithLocalDisk {
    /// Create a request handler
    ///
    /// This method is used to create a request handler. You need to
    /// provide a root directory, and files under this root directory will be
    /// mapped to the request.
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: PathBuf::from(root_dir),
        }
    }
}

impl RequestHandlerFactory for RequestHandlerWithLocalDisk {
    fn request(&self, request: &Request) -> Option<Box<dyn RequestHandler>> {
        let url = if request.url.len() == 0 {
            "http://localhost/index.html"
        } else {
            &request.url
        };

        let mut path = Url::parse(url).ok()?.path().to_string();
        if path.starts_with("/") {
            path = path[1..].to_string();
        }

        Some(Box::new(LocalDiskRequestHandler::new(
            self.root_dir.join(path),
        )))
    }
}

/// Request information
#[derive(Debug)]
pub struct Request<'a> {
    /// Request URL
    pub url: &'a str,
    /// Request method
    pub method: &'a str,
    /// Request referrer
    pub referrer: &'a str,
}

impl<'a> Request<'a> {
    fn from_raw_ptr(request: *mut sys::Request) -> Option<Self> {
        let request = unsafe { &*request };

        Some(Self {
            url: unsafe { CStr::from_ptr(request.url).to_str().ok()? },
            method: unsafe { CStr::from_ptr(request.method).to_str().ok()? },
            referrer: unsafe { CStr::from_ptr(request.referrer).to_str().ok()? },
        })
    }
}

/// Response information
#[repr(C)]
#[derive(Debug)]
pub struct Response {
    /// Response status code
    pub status_code: u32,
    /// Response content length
    pub content_length: u64,
    /// Response MIME type
    pub mime_type: String,
}

/// Request handler
///
/// This is mainly used to handle requests. You can implement custom request
/// handling through this interface.
pub trait RequestHandler: Send + Sync {
    /// Open request
    ///
    /// This method is used to open a request. You can open files, network
    /// resources, etc. in this method.
    ///
    /// If opening fails, return `false`, otherwise return `true`.
    ///
    /// This method is generally called first.
    fn open(&mut self) -> bool;

    /// Get response
    ///
    /// This method is used to get the response. You can return response
    /// content, status code, etc. in this method.
    ///
    /// If getting fails, return `None`, otherwise return `Some(Response)`.
    ///
    /// This method is generally called after the `open` method.
    fn get_response(&mut self) -> Option<Response>;

    /// Skip content
    ///
    /// This method is used to skip response content. You can skip response
    /// content in this method.
    ///
    /// If skipping fails, return `None`, otherwise return `Some(usize)`, and
    /// the returned length is the skipped length.
    ///
    /// This method is generally called after the `open` method.
    fn skip(&mut self, size: usize) -> Option<usize>;

    /// Read response
    ///
    /// This method is used to read the response. You can read response content
    /// in this method.
    ///
    /// If reading fails, return `None`, otherwise return `Some(usize)`, and the
    /// returned length is the read length.
    ///
    /// This method is generally called after the `open` method.
    fn read(&mut self, buffer: &mut [u8]) -> Option<usize>;

    /// Cancel request
    ///
    /// This method is used to cancel the request. You can cancel the request in
    /// this method. When the request ends, this method will be called.
    fn cancel(&mut self);
}

/// Implement request handling for dynamic types
impl RequestHandler for Box<dyn RequestHandler> {
    fn open(&mut self) -> bool {
        self.as_mut().open()
    }

    fn get_response(&mut self) -> Option<Response> {
        self.as_mut().get_response()
    }

    fn skip(&mut self, size: usize) -> Option<usize> {
        self.as_mut().skip(size)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        self.as_mut().read(buffer)
    }

    fn cancel(&mut self) {
        self.as_mut().cancel()
    }
}

/// Custom Scheme handler factory
///
/// This interface is used to handle custom Scheme requests.
pub trait RequestHandlerFactory: Send + Sync {
    /// Handle request
    ///
    /// This method is used to handle requests. You can return request handling
    /// in this method.
    ///
    /// If you don't handle this request, return `None`, otherwise return a
    /// request handler.
    fn request(&self, request: &Request) -> Option<Box<dyn RequestHandler>>;
}

/// Custom Scheme attributes
pub struct CustomSchemeAttributes {
    pub(crate) name: CString,
    pub(crate) domain: CString,
    pub(crate) handler: CustomRequestHandlerFactory,
}

impl<'a> CustomSchemeAttributes {
    /// Create custom Scheme attributes
    ///
    /// This method is used to create custom Scheme attributes. You need to
    /// provide the Scheme name, domain, and handler.
    ///
    /// The name is the Scheme name, the domain is the Scheme domain, and the
    /// handler is the program used to handle requests.
    pub fn new(name: &'a str, domain: &'a str, handler: CustomRequestHandlerFactory) -> Self {
        Self {
            domain: CString::new(domain).unwrap(),
            name: CString::new(name).unwrap(),
            handler,
        }
    }
}

struct ICustomRequestHandlerFactory {
    raw: ThreadSafePointer<Box<dyn RequestHandlerFactory>>,
    raw_handler: ThreadSafePointer<sys::RequestHandlerFactory>,
}

impl Drop for ICustomRequestHandlerFactory {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.raw.as_ptr()) });
    }
}

/// Custom Scheme handler
///
/// This struct is used to handle custom Scheme requests.
#[derive(Clone)]
pub struct CustomRequestHandlerFactory(Arc<ICustomRequestHandlerFactory>);

impl CustomRequestHandlerFactory {
    pub fn new<T>(handler: T) -> Self
    where
        T: RequestHandlerFactory + 'static,
    {
        let raw: *mut Box<dyn RequestHandlerFactory> = Box::into_raw(Box::new(Box::new(handler)));
        let raw_handler = Box::into_raw(Box::new(sys::RequestHandlerFactory {
            request: Some(on_create_request_handler),
            destroy_request_handler: Some(on_destroy_request_handler),
            context: raw as _,
        }));

        Self(Arc::new(ICustomRequestHandlerFactory {
            raw: ThreadSafePointer::new(raw),
            raw_handler: ThreadSafePointer::new(raw_handler),
        }))
    }

    pub(crate) fn as_raw_handler(&self) -> &ThreadSafePointer<sys::RequestHandlerFactory> {
        &self.0.raw_handler
    }
}

/// Used to get the MIME type of a file
fn get_mime_type(path: &Path) -> Option<String> {
    Some(
        mime_guess::from_ext(path.extension()?.to_str()?)
            .first()?
            .to_string(),
    )
}

extern "C" fn on_create_request_handler(
    request: *mut sys::Request,
    context: *mut c_void,
) -> *mut sys::RequestHandler {
    if request.is_null() {
        return null_mut();
    }

    if let Some(request) = Request::from_raw_ptr(request) {
        if let Some(handler) =
            unsafe { &*(context as *mut Box<dyn RequestHandlerFactory>) }.request(&request)
        {
            return Box::into_raw(Box::new(sys::RequestHandler {
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

// This is to destroy `RequestHandler`, not to destroy `SchemeHandlerFactory`.
extern "C" fn on_destroy_request_handler(handler: *mut sys::RequestHandler) {
    drop(unsafe { Box::from_raw(handler) });
}

extern "C" fn on_open(context: *mut c_void) -> bool {
    unsafe { &mut *(context as *mut Box<dyn RequestHandler>) }.open()
}

extern "C" fn on_get_response(response: *mut sys::Response, context: *mut c_void) {
    let response = unsafe { &mut *response };

    // Default return 404 response.
    let res = unsafe { &mut *(context as *mut Box<dyn RequestHandler>) }
        .get_response()
        .unwrap_or_else(|| Response {
            status_code: 404,
            content_length: 0,
            mime_type: "text/plain".to_string(),
        });

    {
        let mime_type_bytes = res.mime_type.as_bytes();
        let mime_type_len = mime_type_bytes.len().min(254);

        unsafe {
            std::ptr::copy_nonoverlapping(
                mime_type_bytes.as_ptr(),
                response.mime_type as *mut u8,
                mime_type_len,
            );

            *(response.mime_type.add(mime_type_len) as *mut u8) = 0;
        }
    }

    response.status_code = res.status_code as i32;
    response.content_length = res.content_length;
}

extern "C" fn on_skip(size: usize, skip_bytes: *mut i32, context: *mut c_void) -> bool {
    let skip_bytes = unsafe { &mut *skip_bytes };

    if let Some(len) = unsafe { &mut *(context as *mut Box<dyn RequestHandler>) }.skip(size) {
        *skip_bytes = len as i32;

        true
    } else {
        // If skipping fails, set |skip_bytes| to -2 and return false.
        *skip_bytes = -2;

        false
    }
}

extern "C" fn on_read(
    buffer: *mut u8,
    size: usize,
    read_bytes: *mut i32,
    context: *mut c_void,
) -> bool {
    let read_bytes = unsafe { &mut *read_bytes };

    if let Some(len) = unsafe { &mut *(context as *mut Box<dyn RequestHandler>) }
        .read(unsafe { std::slice::from_raw_parts_mut(buffer, size) })
    {
        *read_bytes = len as i32;

        // If the end of the response is reached, return false.
        len > 0
    } else {
        // If reading fails, set |read_bytes| to -2 and return false.
        *read_bytes = -2;

        false
    }
}

extern "C" fn on_cancel(context: *mut c_void) {
    unsafe { &mut *(context as *mut Box<dyn RequestHandler>) }.cancel();
}

// Destroy `RequestHandler`
extern "C" fn on_destroy(context: *mut c_void) {
    drop(unsafe { Box::from_raw(context as *mut Box<dyn RequestHandler>) });
}
