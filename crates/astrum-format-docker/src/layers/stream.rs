use std::cell::Cell;
use std::io::{self, BufRead, BufReader, Chain, Cursor, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use flate2::bufread::DeflateDecoder;
use ruzstd::decoding::errors::{FrameDecoderError, ReadFrameHeaderError};
use ruzstd::decoding::{BlockDecodingStrategy, FrameDecoder};

use super::limits::Limits;

const BUFFER: usize = 64 * 1024;
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];
const GZIP_FOOTER: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    Output,
    Ratio,
    Frames,
    Window,
    Cancelled,
}

#[derive(Default)]
pub struct Watch {
    cut: Cell<Option<Cut>>,
    input: Cell<u64>,
    source_failed: Cell<bool>,
}

impl Watch {
    pub fn cut(&self) -> Option<Cut> {
        self.cut.get()
    }

    pub fn source_failed(&self) -> bool {
        self.source_failed.get()
    }

    fn set_cut(&self, cut: Cut) {
        if self.cut.get().is_none() {
            self.cut.set(Some(cut));
        }
    }
}

struct Counted<R> {
    inner: R,
    watch: Rc<Watch>,
}

impl<R: Read> Read for Counted<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(read) => {
                self.watch
                    .input
                    .set(self.watch.input.get().saturating_add(read as u64));
                Ok(read)
            }
            Err(error) => {
                self.watch.source_failed.set(true);
                Err(error)
            }
        }
    }
}

type Source<R> = BufReader<Chain<Cursor<Vec<u8>>, Counted<R>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Compression {
    Plain,
    Gzip,
    Zstd,
}

fn detect(head: &[u8]) -> Compression {
    let skippable =
        head.len() >= 4 && (0x50..=0x5f).contains(&head[0]) && head[1..4] == [0x2a, 0x4d, 0x18];
    if head.starts_with(&GZIP_MAGIC) {
        Compression::Gzip
    } else if head.starts_with(&ZSTD_MAGIC) || skippable {
        Compression::Zstd
    } else {
        Compression::Plain
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn skip(source: &mut impl Read, length: u64) -> io::Result<()> {
    let copied = io::copy(&mut source.take(length), &mut io::sink())?;
    match copied == length {
        true => Ok(()),
        false => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the archive ended inside a skipped field",
        )),
    }
}

fn skip_text(source: &mut impl BufRead, room: usize) -> io::Result<usize> {
    let mut used = 0usize;
    loop {
        let buffer = source.fill_buf()?;
        if buffer.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        let (take, found) = match buffer.iter().position(|byte| *byte == 0) {
            Some(index) => (index + 1, true),
            None => (buffer.len(), false),
        };
        used += take;
        if used > room {
            return Err(invalid("a gzip header field is too long"));
        }
        source.consume(take);
        if found {
            return Ok(used);
        }
    }
}

fn gzip_header(source: &mut impl BufRead, limit: usize) -> io::Result<()> {
    let mut fixed = [0u8; 10];
    source.read_exact(&mut fixed)?;
    if fixed[..2] != GZIP_MAGIC || fixed[2] != 8 || fixed[3] & 0xe0 != 0 {
        return Err(invalid("the data is not a gzip member"));
    }
    let flags = fixed[3];
    let mut used = fixed.len();
    if flags & 4 != 0 {
        let mut length = [0u8; 2];
        source.read_exact(&mut length)?;
        let length = u16::from_le_bytes(length);
        skip(source, u64::from(length))?;
        used += 2 + usize::from(length);
    }
    for flag in [8u8, 16u8] {
        if flags & flag != 0 {
            used += skip_text(source, limit.saturating_sub(used))?;
        }
    }
    if flags & 2 != 0 {
        skip(source, 2)?;
    }
    Ok(())
}

enum GzipState<R> {
    Header(Source<R>),
    Body(DeflateDecoder<Source<R>>),
    Done,
}

struct Gzip<R> {
    state: GzipState<R>,
    members: u64,
    limits: Limits,
    watch: Rc<Watch>,
}

impl<R: Read> Read for Gzip<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            match std::mem::replace(&mut self.state, GzipState::Done) {
                GzipState::Done => return Ok(0),
                GzipState::Header(mut source) => {
                    if source.fill_buf()?.is_empty() {
                        return Ok(0);
                    }
                    if self.members >= self.limits.max_frames {
                        self.watch.set_cut(Cut::Frames);
                        return Ok(0);
                    }
                    self.members += 1;
                    gzip_header(&mut source, self.limits.max_header)?;
                    self.state = GzipState::Body(DeflateDecoder::new(source));
                }
                GzipState::Body(mut body) => {
                    let read = body.read(buf)?;
                    if read > 0 {
                        self.state = GzipState::Body(body);
                        return Ok(read);
                    }
                    let mut source = body.into_inner();
                    let mut footer = [0u8; GZIP_FOOTER];
                    source.read_exact(&mut footer)?;
                    self.state = GzipState::Header(source);
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameState {
    Between,
    Inside,
    Done,
}

struct Zstd<R> {
    source: Source<R>,
    decoder: FrameDecoder,
    state: FrameState,
    frames: u64,
    limits: Limits,
    watch: Rc<Watch>,
}

impl<R: Read> Read for Zstd<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            match self.state {
                FrameState::Done => return Ok(0),
                FrameState::Between => {
                    if self.source.fill_buf()?.is_empty() {
                        self.state = FrameState::Done;
                        return Ok(0);
                    }
                    if self.frames >= self.limits.max_frames {
                        self.watch.set_cut(Cut::Frames);
                        self.state = FrameState::Done;
                        return Ok(0);
                    }
                    self.frames += 1;
                    match self.decoder.init(&mut self.source) {
                        Ok(()) => self.state = FrameState::Inside,
                        Err(FrameDecoderError::ReadFrameHeaderError(
                            ReadFrameHeaderError::SkipFrame { length, .. },
                        )) => skip(&mut self.source, u64::from(length))?,
                        Err(FrameDecoderError::WindowSizeTooBig { .. }) => {
                            self.watch.set_cut(Cut::Window);
                            self.state = FrameState::Done;
                            return Ok(0);
                        }
                        Err(error) => {
                            self.state = FrameState::Done;
                            return Err(invalid(error));
                        }
                    }
                }
                FrameState::Inside => {
                    while self.decoder.can_collect() == 0 && !self.decoder.is_finished() {
                        self.decoder
                            .decode_blocks(&mut self.source, BlockDecodingStrategy::UptoBlocks(1))
                            .map_err(invalid)?;
                    }
                    if self.decoder.can_collect() == 0 {
                        self.state = FrameState::Between;
                        continue;
                    }
                    return self.decoder.read(buf);
                }
            }
        }
    }
}

enum Decoder<R> {
    Plain(Source<R>),
    Gzip(Box<Gzip<R>>),
    Zstd(Box<Zstd<R>>),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Decoder::Plain(source) => source.read(buf),
            Decoder::Gzip(gzip) => gzip.read(buf),
            Decoder::Zstd(zstd) => zstd.read(buf),
        }
    }
}

pub struct Bounded<R> {
    inner: Decoder<R>,
    watch: Rc<Watch>,
    limits: Limits,
    output: u64,
    cancel: Arc<AtomicBool>,
}

impl<R: Read> Read for Bounded<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() || self.watch.cut().is_some() {
            return Ok(0);
        }
        if self.cancel.load(Ordering::Relaxed) {
            self.watch.set_cut(Cut::Cancelled);
            return Ok(0);
        }
        let room = self.limits.max_output - self.output;
        let ask =
            usize::try_from(room.saturating_add(1)).map_or(buf.len(), |ask| ask.min(buf.len()));
        let read = self.inner.read(&mut buf[..ask])?;
        if read as u64 > room {
            self.output = self.limits.max_output;
            self.watch.set_cut(Cut::Output);
            return Ok(usize::try_from(room).unwrap_or(0));
        }
        self.output += read as u64;
        let input = self.watch.input.get().max(1);
        if self.output >= self.limits.ratio_floor && self.output / input > self.limits.max_ratio {
            self.watch.set_cut(Cut::Ratio);
        }
        Ok(read)
    }
}

pub struct Layer<R> {
    pub reader: Bounded<R>,
    pub watch: Rc<Watch>,
}

pub fn open<R: Read>(source: R, limits: Limits, cancel: Arc<AtomicBool>) -> io::Result<Layer<R>> {
    let watch = Rc::new(Watch::default());
    let mut counted = Counted {
        inner: source,
        watch: watch.clone(),
    };
    let mut head = Vec::with_capacity(ZSTD_MAGIC.len());
    (&mut counted)
        .take(ZSTD_MAGIC.len() as u64)
        .read_to_end(&mut head)?;
    let kind = detect(&head);
    let source = BufReader::with_capacity(BUFFER, Cursor::new(head).chain(counted));
    let inner = match kind {
        Compression::Plain => Decoder::Plain(source),
        Compression::Gzip => Decoder::Gzip(Box::new(Gzip {
            state: GzipState::Header(source),
            members: 0,
            limits,
            watch: watch.clone(),
        })),
        Compression::Zstd => {
            let mut decoder = FrameDecoder::new();
            decoder.set_max_window_size(limits.max_window);
            Decoder::Zstd(Box::new(Zstd {
                source,
                decoder,
                state: FrameState::Between,
                frames: 0,
                limits,
                watch: watch.clone(),
            }))
        }
    };
    Ok(Layer {
        reader: Bounded {
            inner,
            watch: watch.clone(),
            limits,
            output: 0,
            cancel,
        },
        watch,
    })
}
