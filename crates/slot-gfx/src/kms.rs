use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

use libloading::Library;

use crate::surface::{GfxError, Surface};

const CARDS: [&str; 4] = [
    "/dev/dri/card0",
    "/dev/dri/card1",
    "/dev/dri/card2",
    "/dev/dri/card3",
];

const DRM_MODE_CONNECTED: c_int = 1;
const DRM_MODE_TYPE_PREFERRED: u32 = 1 << 3;
const DRM_MODE_PAGE_FLIP_EVENT: u32 = 0x01;
const DRM_EVENT_CONTEXT_VERSION: c_int = 2;

const GBM_FORMAT_XRGB8888: u32 = 0x3432_5258;
const GBM_BO_USE_SCANOUT: u32 = 1 << 0;
const GBM_BO_USE_RENDERING: u32 = 1 << 2;

const EGL_NONE: i32 = 0x3038;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_NATIVE_VISUAL_ID: i32 = 0x302E;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_WINDOW_BIT: i32 = 0x0004;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;

const POLLIN: i16 = 0x001;

type Ptr = *mut c_void;

#[repr(C)]
#[derive(Copy, Clone)]
struct ModeInfo {
    clock: u32,
    hdisplay: u16,
    hsync_start: u16,
    hsync_end: u16,
    htotal: u16,
    hskew: u16,
    vdisplay: u16,
    vsync_start: u16,
    vsync_end: u16,
    vtotal: u16,
    vscan: u16,
    vrefresh: u32,
    flags: u32,
    kind: u32,
    name: [c_char; 32],
}

#[repr(C)]
struct Resources {
    count_fbs: c_int,
    fbs: *mut u32,
    count_crtcs: c_int,
    crtcs: *mut u32,
    count_connectors: c_int,
    connectors: *mut u32,
    count_encoders: c_int,
    encoders: *mut u32,
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

#[repr(C)]
struct Connector {
    connector_id: u32,
    encoder_id: u32,
    connector_type: u32,
    connector_type_id: u32,
    connection: c_int,
    mm_width: u32,
    mm_height: u32,
    subpixel: c_int,
    count_modes: c_int,
    modes: *mut ModeInfo,
    count_props: c_int,
    props: *mut u32,
    prop_values: *mut u64,
    count_encoders: c_int,
    encoders: *mut u32,
}

#[repr(C)]
struct Encoder {
    encoder_id: u32,
    encoder_type: u32,
    crtc_id: u32,
    possible_crtcs: u32,
    possible_clones: u32,
}

#[repr(C)]
struct Crtc {
    crtc_id: u32,
    buffer_id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    mode_valid: c_int,
    mode: ModeInfo,
    gamma_size: c_int,
}

type FlipHandler = extern "C" fn(c_int, c_uint, c_uint, c_uint, Ptr);

#[repr(C)]
struct EventContext {
    version: c_int,
    vblank_handler: Option<FlipHandler>,
    page_flip_handler: Option<FlipHandler>,
}

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}

extern "C" {
    fn poll(fds: *mut PollFd, nfds: u64, timeout: c_int) -> c_int;
}

struct Drm {
    get_resources: unsafe extern "C" fn(c_int) -> *mut Resources,
    free_resources: unsafe extern "C" fn(*mut Resources),
    get_connector: unsafe extern "C" fn(c_int, u32) -> *mut Connector,
    free_connector: unsafe extern "C" fn(*mut Connector),
    get_encoder: unsafe extern "C" fn(c_int, u32) -> *mut Encoder,
    free_encoder: unsafe extern "C" fn(*mut Encoder),
    get_crtc: unsafe extern "C" fn(c_int, u32) -> *mut Crtc,
    free_crtc: unsafe extern "C" fn(*mut Crtc),
    add_fb: unsafe extern "C" fn(c_int, u32, u32, u8, u8, u32, u32, *mut u32) -> c_int,
    rm_fb: unsafe extern "C" fn(c_int, u32) -> c_int,
    set_crtc:
        unsafe extern "C" fn(c_int, u32, u32, u32, u32, *const u32, c_int, *const ModeInfo) -> c_int,
    page_flip: unsafe extern "C" fn(c_int, u32, u32, u32, Ptr) -> c_int,
    handle_event: unsafe extern "C" fn(c_int, *mut EventContext) -> c_int,
    _lib: Library,
}

struct Gbm {
    create_device: unsafe extern "C" fn(c_int) -> Ptr,
    device_destroy: unsafe extern "C" fn(Ptr),
    surface_create: unsafe extern "C" fn(Ptr, u32, u32, u32, u32) -> Ptr,
    surface_destroy: unsafe extern "C" fn(Ptr),
    lock_front_buffer: unsafe extern "C" fn(Ptr) -> Ptr,
    release_buffer: unsafe extern "C" fn(Ptr, Ptr),
    bo_get_handle: unsafe extern "C" fn(Ptr) -> u64,
    bo_get_stride: unsafe extern "C" fn(Ptr) -> u32,
    _lib: Library,
}

struct Egl {
    get_display: unsafe extern "C" fn(Ptr) -> Ptr,
    initialize: unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32,
    bind_api: unsafe extern "C" fn(u32) -> u32,
    choose_config: unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32,
    get_config_attrib: unsafe extern "C" fn(Ptr, Ptr, i32, *mut i32) -> u32,
    create_window_surface: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr,
    create_context: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr,
    make_current: unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32,
    swap_buffers: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    get_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    get_error: unsafe extern "C" fn() -> i32,
    terminate: unsafe extern "C" fn(Ptr) -> u32,
    _lib: Library,
}

fn open(names: &[&str]) -> Result<Library, GfxError> {
    let mut last = String::new();
    for name in names {
        match unsafe { Library::new(name) } {
            Ok(lib) => return Ok(lib),
            Err(e) => last = format!("{name}: {e}"),
        }
    }
    Err(GfxError::Context(last))
}

unsafe fn sym<T: Copy>(lib: &Library, name: &str) -> Result<T, GfxError> {
    lib.get::<T>(name.as_bytes())
        .map(|s| *s)
        .map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

impl Drm {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libdrm.so.2", "libdrm.so"])?;
        unsafe {
            Ok(Drm {
                get_resources: sym(&lib, "drmModeGetResources")?,
                free_resources: sym(&lib, "drmModeFreeResources")?,
                get_connector: sym(&lib, "drmModeGetConnector")?,
                free_connector: sym(&lib, "drmModeFreeConnector")?,
                get_encoder: sym(&lib, "drmModeGetEncoder")?,
                free_encoder: sym(&lib, "drmModeFreeEncoder")?,
                get_crtc: sym(&lib, "drmModeGetCrtc")?,
                free_crtc: sym(&lib, "drmModeFreeCrtc")?,
                add_fb: sym(&lib, "drmModeAddFB")?,
                rm_fb: sym(&lib, "drmModeRmFB")?,
                set_crtc: sym(&lib, "drmModeSetCrtc")?,
                page_flip: sym(&lib, "drmModePageFlip")?,
                handle_event: sym(&lib, "drmHandleEvent")?,
                _lib: lib,
            })
        }
    }
}

impl Gbm {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libgbm.so.1", "libgbm.so", "libmali.so"])?;
        unsafe {
            Ok(Gbm {
                create_device: sym(&lib, "gbm_create_device")?,
                device_destroy: sym(&lib, "gbm_device_destroy")?,
                surface_create: sym(&lib, "gbm_surface_create")?,
                surface_destroy: sym(&lib, "gbm_surface_destroy")?,
                lock_front_buffer: sym(&lib, "gbm_surface_lock_front_buffer")?,
                release_buffer: sym(&lib, "gbm_surface_release_buffer")?,
                bo_get_handle: sym(&lib, "gbm_bo_get_handle")?,
                bo_get_stride: sym(&lib, "gbm_bo_get_stride")?,
                _lib: lib,
            })
        }
    }
}

impl Egl {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libEGL.so.1", "libEGL.so", "libmali.so"])?;
        unsafe {
            Ok(Egl {
                get_display: sym(&lib, "eglGetDisplay")?,
                initialize: sym(&lib, "eglInitialize")?,
                bind_api: sym(&lib, "eglBindAPI")?,
                choose_config: sym(&lib, "eglChooseConfig")?,
                get_config_attrib: sym(&lib, "eglGetConfigAttrib")?,
                create_window_surface: sym(&lib, "eglCreateWindowSurface")?,
                create_context: sym(&lib, "eglCreateContext")?,
                make_current: sym(&lib, "eglMakeCurrent")?,
                swap_buffers: sym(&lib, "eglSwapBuffers")?,
                get_proc_address: sym(&lib, "eglGetProcAddress")?,
                get_error: sym(&lib, "eglGetError")?,
                terminate: sym(&lib, "eglTerminate")?,
                _lib: lib,
            })
        }
    }

    fn fail(&self, what: &str) -> GfxError {
        crate::fbdev::egl_error(what, unsafe { (self.get_error)() })
    }
}

struct Output {
    connector: u32,
    crtc: u32,
    mode: ModeInfo,
    saved: *mut Crtc,
}

fn pick_mode(conn: &Connector) -> Option<ModeInfo> {
    if conn.count_modes <= 0 || conn.modes.is_null() {
        return None;
    }
    let modes = unsafe { std::slice::from_raw_parts(conn.modes, conn.count_modes as usize) };
    modes
        .iter()
        .find(|m| m.kind & DRM_MODE_TYPE_PREFERRED != 0)
        .or_else(|| modes.iter().max_by_key(|m| u32::from(m.hdisplay) * u32::from(m.vdisplay)))
        .copied()
}

fn find_output(drm: &Drm, fd: c_int) -> Option<Output> {
    unsafe {
        let res = (drm.get_resources)(fd);
        if res.is_null() {
            return None;
        }
        let r = &*res;
        let ids = |p: *mut u32, n: c_int| -> Vec<u32> {
            if p.is_null() || n <= 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(p, n as usize).to_vec()
            }
        };
        let connectors = ids(r.connectors, r.count_connectors);
        let crtcs = ids(r.crtcs, r.count_crtcs);
        (drm.free_resources)(res);
        for id in connectors {
            let conn = (drm.get_connector)(fd, id);
            if conn.is_null() {
                continue;
            }
            let c = &*conn;
            let found = (c.connection == DRM_MODE_CONNECTED)
                .then(|| pick_mode(c))
                .flatten()
                .and_then(|mode| {
                    let mut encoders = vec![c.encoder_id];
                    encoders.extend(ids(c.encoders, c.count_encoders));
                    encoders.into_iter().filter(|e| *e != 0).find_map(|e| {
                        let enc = (drm.get_encoder)(fd, e);
                        if enc.is_null() {
                            return None;
                        }
                        let en = &*enc;
                        let crtc = if en.crtc_id != 0 {
                            Some(en.crtc_id)
                        } else {
                            crtcs
                                .iter()
                                .enumerate()
                                .find(|(i, _)| en.possible_crtcs & (1 << i) != 0)
                                .map(|(_, id)| *id)
                        };
                        (drm.free_encoder)(enc);
                        crtc.map(|crtc| (crtc, mode))
                    })
                });
            (drm.free_connector)(conn);
            if let Some((crtc, mode)) = found {
                return Some(Output {
                    connector: id,
                    crtc,
                    mode,
                    saved: (drm.get_crtc)(fd, crtc),
                });
            }
        }
        None
    }
}

extern "C" fn flipped(_fd: c_int, _seq: c_uint, _sec: c_uint, _usec: c_uint, data: Ptr) {
    unsafe { *(data as *mut bool) = false };
}

pub struct KmsSurface {
    egl: Egl,
    gles: Library,
    gbm: Gbm,
    drm: Drm,
    card: File,
    output: Output,
    gbm_device: Ptr,
    gbm_surface: Ptr,
    display: Ptr,
    surface: Ptr,
    context: Ptr,
    size: (u32, u32),
    shown: Option<Ptr>,
    fbs: Vec<(Ptr, u32)>,
    flip_pending: Box<bool>,
}

impl KmsSurface {
    pub fn available() -> bool {
        CARDS.iter().any(|c| std::path::Path::new(c).exists())
    }

    pub fn new() -> Result<Self, GfxError> {
        let drm = Drm::load()?;
        let (card, output) = CARDS
            .iter()
            .filter_map(|path| {
                let card = OpenOptions::new().read(true).write(true).open(path).ok()?;
                let output = find_output(&drm, card.as_raw_fd())?;
                eprintln!(
                    "slot: kms {path} connector {} crtc {} mode {}x{}@{}",
                    output.connector,
                    output.crtc,
                    output.mode.hdisplay,
                    output.mode.vdisplay,
                    output.mode.vrefresh
                );
                Some((card, output))
            })
            .next()
            .ok_or_else(|| GfxError::Context("kms: no connected display on /dev/dri".into()))?;
        let size = (
            u32::from(output.mode.hdisplay),
            u32::from(output.mode.vdisplay),
        );
        let gbm = Gbm::load()?;
        let egl = Egl::load()?;
        let gles = open(&["libGLESv2.so.2", "libGLESv2.so", "libmali.so"])?;
        unsafe {
            let gbm_device = (gbm.create_device)(card.as_raw_fd());
            if gbm_device.is_null() {
                return Err(GfxError::Context("gbm_create_device failed".into()));
            }
            let gbm_surface = (gbm.surface_create)(
                gbm_device,
                size.0,
                size.1,
                GBM_FORMAT_XRGB8888,
                GBM_BO_USE_SCANOUT | GBM_BO_USE_RENDERING,
            );
            if gbm_surface.is_null() {
                return Err(GfxError::Context("gbm_surface_create failed".into()));
            }
            let display = (egl.get_display)(gbm_device);
            if display.is_null() {
                return Err(egl.fail("eglGetDisplay"));
            }
            if (egl.initialize)(display, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                return Err(egl.fail("eglInitialize"));
            }
            if (egl.bind_api)(EGL_OPENGL_ES_API) == 0 {
                return Err(egl.fail("eglBindAPI"));
            }
            let config = choose_config(&egl, display)?;
            let surface =
                (egl.create_window_surface)(display, config, gbm_surface, std::ptr::null());
            if surface.is_null() {
                return Err(egl.fail("eglCreateWindowSurface"));
            }
            let context_attrs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
            let context = (egl.create_context)(
                display,
                config,
                std::ptr::null_mut(),
                context_attrs.as_ptr(),
            );
            if context.is_null() {
                return Err(egl.fail("eglCreateContext"));
            }
            if (egl.make_current)(display, surface, surface, context) == 0 {
                return Err(egl.fail("eglMakeCurrent"));
            }
            Ok(KmsSurface {
                egl,
                gles,
                gbm,
                drm,
                card,
                output,
                gbm_device,
                gbm_surface,
                display,
                surface,
                context,
                size,
                shown: None,
                fbs: Vec::new(),
                flip_pending: Box::new(false),
            })
        }
    }

    fn fb_for(&mut self, bo: Ptr) -> Result<u32, GfxError> {
        if let Some((_, fb)) = self.fbs.iter().find(|(b, _)| *b == bo) {
            return Ok(*fb);
        }
        let mut fb = 0;
        let rc = unsafe {
            let handle = (self.gbm.bo_get_handle)(bo) as u32;
            let stride = (self.gbm.bo_get_stride)(bo);
            (self.drm.add_fb)(
                self.card.as_raw_fd(),
                self.size.0,
                self.size.1,
                24,
                32,
                stride,
                handle,
                &mut fb,
            )
        };
        if rc != 0 {
            return Err(GfxError::Context(format!("drmModeAddFB: {rc}")));
        }
        self.fbs.push((bo, fb));
        Ok(fb)
    }

    fn wait_flip(&mut self) {
        let fd = self.card.as_raw_fd();
        let mut ctx = EventContext {
            version: DRM_EVENT_CONTEXT_VERSION,
            vblank_handler: None,
            page_flip_handler: Some(flipped),
        };
        while *self.flip_pending {
            let mut pfd = PollFd {
                fd,
                events: POLLIN,
                revents: 0,
            };
            let ready = unsafe { poll(&mut pfd, 1, 100) };
            if ready <= 0 {
                eprintln!("slot: kms page flip timed out");
                *self.flip_pending = false;
                break;
            }
            unsafe { (self.drm.handle_event)(fd, &mut ctx) };
        }
    }
}

unsafe fn choose_config(egl: &Egl, display: Ptr) -> Result<Ptr, GfxError> {
    let attrs = [
        EGL_SURFACE_TYPE,
        EGL_WINDOW_BIT,
        EGL_RED_SIZE,
        8,
        EGL_GREEN_SIZE,
        8,
        EGL_BLUE_SIZE,
        8,
        EGL_ALPHA_SIZE,
        0,
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,
        EGL_NONE,
    ];
    let mut configs = [std::ptr::null_mut(); 64];
    let mut found = 0;
    if (egl.choose_config)(display, attrs.as_ptr(), configs.as_mut_ptr(), 64, &mut found) == 0 {
        return Err(egl.fail("eglChooseConfig"));
    }
    let configs = &configs[..found.max(0) as usize];
    let matches_gbm = |c: &&Ptr| {
        let mut id = 0;
        (egl.get_config_attrib)(display, **c, EGL_NATIVE_VISUAL_ID, &mut id) != 0
            && id as u32 == GBM_FORMAT_XRGB8888
    };
    configs
        .iter()
        .find(matches_gbm)
        .or_else(|| configs.first())
        .copied()
        .ok_or_else(|| GfxError::Context("eglChooseConfig: no es2 config for gbm".into()))
}

impl Surface for KmsSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        let ok = unsafe {
            (self.egl.make_current)(self.display, self.surface, self.surface, self.context)
        };
        match ok {
            0 => Err(self.egl.fail("eglMakeCurrent")),
            _ => Ok(()),
        }
    }

    fn window_size(&self) -> (u32, u32) {
        self.size
    }

    fn swap(&mut self) -> Result<(), GfxError> {
        if unsafe { (self.egl.swap_buffers)(self.display, self.surface) } == 0 {
            return Err(self.egl.fail("eglSwapBuffers"));
        }
        let bo = unsafe { (self.gbm.lock_front_buffer)(self.gbm_surface) };
        if bo.is_null() {
            return Err(GfxError::Context("gbm_surface_lock_front_buffer failed".into()));
        }
        let fb = self.fb_for(bo)?;
        let fd = self.card.as_raw_fd();
        if self.shown.is_none() {
            let rc = unsafe {
                (self.drm.set_crtc)(
                    fd,
                    self.output.crtc,
                    fb,
                    0,
                    0,
                    &self.output.connector,
                    1,
                    &self.output.mode,
                )
            };
            if rc != 0 {
                return Err(GfxError::Context(format!("drmModeSetCrtc: {rc}")));
            }
        } else {
            *self.flip_pending = true;
            let data = &mut *self.flip_pending as *mut bool as Ptr;
            let rc = unsafe {
                (self.drm.page_flip)(fd, self.output.crtc, fb, DRM_MODE_PAGE_FLIP_EVENT, data)
            };
            if rc != 0 {
                *self.flip_pending = false;
                eprintln!("slot: drmModePageFlip: {rc}");
            }
            self.wait_flip();
        }
        if let Some(old) = self.shown.replace(bo) {
            unsafe { (self.gbm.release_buffer)(self.gbm_surface, old) };
        }
        Ok(())
    }

    fn proc_address(&self, name: &str) -> *const c_void {
        let Ok(c) = CString::new(name) else {
            return std::ptr::null();
        };
        let exported = unsafe { self.gles.get::<unsafe extern "C" fn()>(name.as_bytes()) };
        match exported {
            Ok(f) => *f as *const c_void,
            Err(_) => unsafe { (self.egl.get_proc_address)(c.as_ptr()) },
        }
    }
}

impl Drop for KmsSurface {
    fn drop(&mut self) {
        let fd = self.card.as_raw_fd();
        unsafe {
            self.wait_flip();
            if !self.output.saved.is_null() {
                let s = &*self.output.saved;
                (self.drm.set_crtc)(
                    fd,
                    s.crtc_id,
                    s.buffer_id,
                    s.x,
                    s.y,
                    &self.output.connector,
                    1,
                    &s.mode,
                );
                (self.drm.free_crtc)(self.output.saved);
            }
            for (_, fb) in self.fbs.drain(..) {
                (self.drm.rm_fb)(fd, fb);
            }
            (self.egl.make_current)(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            (self.egl.terminate)(self.display);
            if let Some(bo) = self.shown.take() {
                (self.gbm.release_buffer)(self.gbm_surface, bo);
            }
            (self.gbm.surface_destroy)(self.gbm_surface);
            (self.gbm.device_destroy)(self.gbm_device);
        }
    }
}

pub fn device_surface() -> Result<Box<dyn Surface>, GfxError> {
    let want = std::env::var("SLOT_GFX").unwrap_or_default();
    if want != "fbdev" && KmsSurface::available() {
        match KmsSurface::new() {
            Ok(s) => return Ok(Box::new(s)),
            Err(e) if want == "kms" => return Err(e),
            Err(e) => eprintln!("slot: kms unavailable, falling back to fbdev: {e}"),
        }
    }
    Ok(Box::new(crate::fbdev::FbdevSurface::new()?))
}
