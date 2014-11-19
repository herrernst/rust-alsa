#![crate_type = "lib"]
#![feature(globs, macro_rules)]

extern crate libc;

use std::ptr;

#[allow(dead_code, unused_attributes, bad_style)]
mod ffi;

#[link(name = "asound")]
extern { }

macro_rules! alsa_ok(
    ($e:expr) => (
        {
            let err = $e;
            if err < 0 {
                return Err(err as int)
            }
            err
        }
    )
)

pub struct PCM<State> {
    i: *mut ffi::snd_pcm_t,
    data: State
}

pub struct Open { #[allow(dead_code)] no_constr: () }
pub struct Prepared {
    channels: uint,
    sample_fmt: Format
}

pub enum Stream {
    Playback,
    Capture
}

impl Stream {
    fn to_ffi(self) -> ffi::snd_pcm_stream_t {
        match self {
            Stream::Playback => ffi::SND_PCM_STREAM_PLAYBACK,
            Stream::Capture  => ffi::SND_PCM_STREAM_CAPTURE
        }
    }
}

pub enum Mode {
    Blocking,
    Nonblocking,
    Asynchronous
}

impl Mode {
    fn to_ffi(self) -> i32 {
        match self {
            Mode::Blocking => 0,
            Mode::Nonblocking => ffi::SND_PCM_NONBLOCK,
            Mode::Asynchronous => ffi::SND_PCM_ASYNC
        }
    }
}

pub enum Access {
    Interleaved,
    Noninterleaved
}

impl Access {
    fn to_ffi(self) -> ffi::snd_pcm_access_t {
        match self {
            Access::Interleaved => ffi::SND_PCM_ACCESS_RW_INTERLEAVED,
            Access::Noninterleaved => ffi::SND_PCM_ACCESS_RW_NONINTERLEAVED
        }
    }
}

pub enum Format {
    Unsigned8,
    Signed16,
    FloatLE
}

impl Format {
    fn to_ffi(self) -> ffi::snd_pcm_format_t {
        match self {
            Format::Unsigned8 => ffi::SND_PCM_FORMAT_U8,
            Format::Signed16 => ffi::SND_PCM_FORMAT_S16,
            Format::FloatLE => ffi::SND_PCM_FORMAT_FLOAT_LE
        }
    }

    fn size(self) -> uint {
        use std::mem::size_of;
        match self {
            Format::Unsigned8 => 1,
            Format::Signed16 => 2,
            Format::FloatLE => size_of::<libc::c_float>()
        }
    }
}

impl PCM<Open> {
    pub fn open(name: &str, stream: Stream, mode: Mode) -> Result<PCM<Open>, int> {
        let mut pcm = PCM {
            i: ptr::null_mut(),
            data: Open { no_constr: () }
        };

        unsafe {
            let name = name.to_c_str();
            alsa_ok!(
                ffi::snd_pcm_open(&mut pcm.i, name.as_ptr(), stream.to_ffi(), mode.to_ffi())
            );
        }

        Ok(pcm)
    }
}

impl PCM<Open> {
    pub fn set_parameters(self, format: Format, access: Access, channels: uint, rate: uint)
        -> Result<PCM<Prepared>, (PCM<Open>, int)>
    {
        unsafe {
            let err = ffi::snd_pcm_set_params(self.i, format.to_ffi(), access.to_ffi(),
                                              channels as u32, rate as u32, 1i32, 500000u32);
            if err < 0 {
                return Err((self, err as int))
            }
        }

        Ok(
            PCM {
                i: self.i,
                data: Prepared {
                    channels: channels,
                    sample_fmt: format
                }
            }
        )
    }

}

impl PCM<Prepared> {
    pub fn write_interleaved<T: Copy>(&mut self, buffer: &[T]) -> Result<uint, int> {
        let channels = self.data.channels;

        assert_eq!(buffer.len() % channels, 0);
        assert_eq!(::std::mem::size_of::<T>(), self.data.sample_fmt.size());

        let n_written = unsafe {
            alsa_ok!(ffi::snd_pcm_writei(self.i, buffer.as_ptr() as *const libc::c_void,
                                         buffer.len() as u64 / channels as u64))
        };

        Ok(n_written as uint)
    }
}
