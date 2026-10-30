use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use libloading::Library;

use super::pacer::Pacer;
use super::ring::Ring;
use super::silence::Silence;
use super::sink::{AudioError, AudioSink};

const SND_PCM_STREAM_PLAYBACK: c_int = 0;
const SND_PCM_FORMAT_S16_LE: c_int = 2;
const SND_PCM_ACCESS_RW_INTERLEAVED: c_int = 3;
const CHANNELS: c_uint = 2;

const LATENCY_US: c_uint = 40_000;

const PERIOD_FRAMES: usize = 512;

const RELEASE_AFTER: Duration = Duration::from_secs(3);

const DEVICES: [&str; 4] = ["plug:default", "default", "plughw:0,0", "hw:0,0"];

const ASOUND: &str = "/proc/asound";

// How often a playing stream looks for a USB card plugged in or pulled out.
const ROUTE_EVERY: Duration = Duration::from_millis(500);

/// The first USB sound card, by number. A USB-C DAC on the R36S comes up as a
/// card of its own, after the codec, and slot plays there while it is in.
/// `default` cannot follow it: ArkOS points `/etc/asound.conf` at the new
/// card, but alsa-lib reads that file once per process.
fn usb_card(asound: &Path) -> Option<u32> {
    let mut cards: Vec<u32> = fs::read_dir(asound)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str()?.strip_prefix("card")?.parse().ok())
        .filter(|n| asound.join(format!("card{n}/usbid")).exists())
        .collect();
    cards.sort_unstable();
    cards.first().copied()
}

type PcmOpen = unsafe extern "C" fn(*mut *mut c_void, *const c_char, c_int, c_int) -> c_int;
type PcmSetParams =
    unsafe extern "C" fn(*mut c_void, c_int, c_int, c_uint, c_uint, c_int, c_uint) -> c_int;
type PcmWritei = unsafe extern "C" fn(*mut c_void, *const c_void, u64) -> i64;
type PcmRecover = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int;
type PcmDrop = unsafe extern "C" fn(*mut c_void) -> c_int;
type PcmClose = unsafe extern "C" fn(*mut c_void) -> c_int;
type StrError = unsafe extern "C" fn(c_int) -> *const c_char;

struct Alsa {
    open: PcmOpen,
    set_params: PcmSetParams,
    writei: PcmWritei,
    recover: PcmRecover,
    drop: PcmDrop,
    close: PcmClose,
    strerror: StrError,
    _lib: Library,
}

impl Alsa {
    fn load() -> Result<Self, AudioError> {
        let lib = unsafe { Library::new("libasound.so.2") }
            .or_else(|_| unsafe { Library::new("libasound.so") })
            .map_err(|e| AudioError::Device(format!("libasound: {e}")))?;
        macro_rules! get {
            ($name:literal) => {
                *lib.get(concat!($name, "\0").as_bytes())
                    .map_err(|e| AudioError::Device(format!("{}: {e}", $name)))?
            };
        }
        unsafe {
            Ok(Alsa {
                open: get!("snd_pcm_open"),
                set_params: get!("snd_pcm_set_params"),
                writei: get!("snd_pcm_writei"),
                recover: get!("snd_pcm_recover"),
                drop: get!("snd_pcm_drop"),
                close: get!("snd_pcm_close"),
                strerror: get!("snd_strerror"),
                _lib: lib,
            })
        }
    }

    fn message(&self, err: c_int) -> String {
        let text = unsafe { (self.strerror)(err) };
        match text.is_null() {
            true => format!("error {err}"),
            false => unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned(),
        }
    }

    fn open_pcm(&self, rate: u32, usb: Option<u32>) -> Result<*mut c_void, AudioError> {
        let mut last = AudioError::NoDevice;
        let usb = usb.map(|n| format!("plughw:{n},0"));
        for name in usb.iter().map(String::as_str).chain(DEVICES) {
            let Ok(cname) = CString::new(name) else {
                continue;
            };
            let mut pcm: *mut c_void = std::ptr::null_mut();
            let err = unsafe { (self.open)(&mut pcm, cname.as_ptr(), SND_PCM_STREAM_PLAYBACK, 0) };
            if err < 0 || pcm.is_null() {
                last = AudioError::Device(format!("{name}: {}", self.message(err)));
                continue;
            }
            let err = unsafe {
                (self.set_params)(
                    pcm,
                    SND_PCM_FORMAT_S16_LE,
                    SND_PCM_ACCESS_RW_INTERLEAVED,
                    CHANNELS,
                    rate,
                    1,
                    LATENCY_US,
                )
            };
            if err < 0 {
                unsafe { (self.close)(pcm) };
                last = AudioError::Config(format!("{name}: {}", self.message(err)));
                continue;
            }
            eprintln!("slot: audio {name} at {rate} Hz");
            return Ok(pcm);
        }
        Err(last)
    }
}

pub struct AlsaSink {
    ring: Arc<Ring>,
    held: Arc<AtomicBool>,
    device: Option<Device>,
}

struct Device {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

impl AlsaSink {
    pub fn new() -> Self {
        AlsaSink {
            ring: Arc::new(Ring::new(0)),
            held: Arc::new(AtomicBool::new(false)),
            device: None,
        }
    }
}

impl Default for AlsaSink {
    fn default() -> Self {
        AlsaSink::new()
    }
}

impl Drop for AlsaSink {
    fn drop(&mut self) {
        self.close();
    }
}

impl AudioSink for AlsaSink {
    fn open(&mut self, sample_rate: u32) -> Result<(), AudioError> {
        self.close();
        let ring = self.ring.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let held = self.held.clone();
        let (ready_tx, ready_rx) = mpsc::channel();
        let join = std::thread::Builder::new()
            .name("slot-audio".into())
            .spawn(move || match play(&ring, sample_rate) {
                Ok(mut device) => {
                    let _ = ready_tx.send(Ok(()));
                    device.run(&ring, &flag, &held);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })
            .map_err(|e| AudioError::Device(e.to_string()))?;
        match ready_rx.recv() {
            Ok(Ok(())) => {
                self.device = Some(Device { stop, join });
                Ok(())
            }
            Ok(Err(e)) => {
                let _ = join.join();
                Err(e)
            }
            Err(_) => {
                let _ = join.join();
                Err(AudioError::Device("output thread stopped".into()))
            }
        }
    }

    fn ring(&self) -> Arc<Ring> {
        self.ring.clone()
    }

    fn close(&mut self) {
        if let Some(d) = self.device.take() {
            d.stop.store(true, Ordering::Relaxed);
            let _ = d.join.join();
        }
    }

    fn hold_open(&self, held: bool) {
        self.held.store(held, Ordering::Relaxed);
    }
}

struct Playback {
    alsa: Alsa,
    pcm: Option<*mut c_void>,
    rate: u32,
    failing: bool,
    usb: Option<u32>,
}

fn play(ring: &Arc<Ring>, sample_rate: u32) -> Result<Playback, AudioError> {
    let alsa = Alsa::load()?;
    ring.reopen(sample_rate);
    Ok(Playback {
        alsa,
        pcm: None,
        rate: sample_rate,
        failing: false,
        usb: None,
    })
}

impl Playback {
    fn run(&mut self, ring: &Ring, stop: &AtomicBool, held: &AtomicBool) {
        let mut buf = vec![0i16; PERIOD_FRAMES * CHANNELS as usize];
        let span = Duration::from_secs_f64(PERIOD_FRAMES as f64 / self.rate as f64);
        let mut silence = Silence::quiet(RELEASE_AFTER);
        let mut pace = Pacer::new(span);
        let route_every = (ROUTE_EVERY.as_secs_f64() / span.as_secs_f64()).ceil() as u32;
        let mut periods = 0u32;
        while !stop.load(Ordering::Relaxed) {
            ring.fill(&mut buf);
            let hold = silence.hear(&buf, span);
            periods += 1;
            if self.pcm.is_some() && periods >= route_every {
                periods = 0;
                let usb = usb_card(Path::new(ASOUND));
                if usb != self.usb {
                    self.release();
                    eprintln!("slot: audio route changed, USB card {usb:?}");
                }
            }
            match (self.pcm, hold) {
                (Some(_), false) if !held.load(Ordering::Relaxed) => {
                    self.release();
                    eprintln!("slot: audio released after silence");
                }
                (None, true) => match self.open() {
                    Ok(pcm) => {
                        eprintln!("slot: audio reopened at {:.1} ms", crate::boot_time::ms());
                        self.pcm = Some(pcm);
                        self.failing = false;
                    }
                    Err(e) => {
                        if !self.failing {
                            eprintln!("slot: audio: {e}, retrying");
                            self.failing = true;
                        }
                    }
                },
                _ => {}
            }
            match self.pcm {
                Some(pcm) => {
                    pace.reset();
                    if !self.write(pcm, &buf) {
                        // A USB card pulled out mid-write: drop it and let
                        // the next period open whatever is left.
                        self.release();
                        std::thread::sleep(span);
                    }
                }
                None => std::thread::sleep(pace.wait(Instant::now())),
            }
        }
    }

    fn open(&mut self) -> Result<*mut c_void, AudioError> {
        self.usb = usb_card(Path::new(ASOUND));
        self.alsa.open_pcm(self.rate, self.usb)
    }

    fn write(&self, pcm: *mut c_void, buf: &[i16]) -> bool {
        let mut written = 0;
        while written < PERIOD_FRAMES {
            let at = written * CHANNELS as usize;
            let frames = unsafe {
                (self.alsa.writei)(
                    pcm,
                    buf[at..].as_ptr() as *const c_void,
                    (PERIOD_FRAMES - written) as u64,
                )
            };
            if frames < 0 {
                let err = unsafe { (self.alsa.recover)(pcm, frames as c_int, 1) };
                if err < 0 {
                    eprintln!("slot: audio: {}", self.alsa.message(err));
                    return false;
                }
                continue;
            }
            written += frames as usize;
        }
        true
    }

    fn release(&mut self) {
        if let Some(pcm) = self.pcm.take() {
            unsafe {
                (self.alsa.drop)(pcm);
                (self.alsa.close)(pcm);
            }
        }
    }
}

impl Drop for Playback {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_usb_card_is_found_after_the_codec() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("card0")).unwrap();
        assert_eq!(usb_card(d.path()), None);
        fs::create_dir_all(d.path().join("card7")).unwrap();
        fs::write(d.path().join("card7/usbid"), "0d8c:0014\n").unwrap();
        fs::create_dir_all(d.path().join("cards")).unwrap();
        assert_eq!(usb_card(d.path()), Some(7));
    }
}
