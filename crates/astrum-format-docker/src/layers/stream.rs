use std::cell::Cell;
use std::io::{self, BufRead, BufReader, Chain, Cursor, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

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
    Frames,
    Window,
    Stalled,
    Deadline,
    Cancelled,
}

impl Cut {
    fn text(self) -> &'static str {
        match self {
            Cut::Output => "the layer output reached its cap",
            Cut::Frames => "the layer has too many compressed members",
            Cut::Window => "the layer asks for too large a window",
            Cut::Stalled => "the layer consumes input without producing output",
            Cut::Deadline => "the layer read passed its deadline",
            Cut::Cancelled => "the layer read was cancelled",
        }
    }
}

#[derive(Clone)]
pub struct Guard {
    cancel: Arc<AtomicBool>,
    deadline: Instant,
}

impl Guard {
    pub fn new(cancel: Arc<AtomicBool>, deadline: Instant) -> Self {
        Guard { cancel, deadline }
    }
}

pub struct Watch {
    cut: Cell<Option<Cut>>,
    input: Cell<u64>,
    output: Cell<u64>,
    mark_input: Cell<u64>,
    mark_output: Cell<u64>,
    source_failed: Cell<bool>,
    guard: Guard,
    limits: Limits,
}

impl Watch {
    fn new(guard: Guard, limits: Limits) -> Self {
        Watch {
            cut: Cell::new(None),
            input: Cell::new(0),
            output: Cell::new(0),
            mark_input: Cell::new(0),
            mark_output: Cell::new(0),
            source_failed: Cell::new(false),
            guard,
            limits,
        }
    }

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

    fn stop(&self, cut: Cut) -> io::Error {
        self.set_cut(cut);
        io::Error::other(cut.text())
    }

    fn poll(&self) -> io::Result<()> {
        if let Some(cut) = self.cut.get() {
            return Err(io::Error::other(cut.text()));
        }
        if self.guard.cancel.load(Ordering::Relaxed) {
            return Err(self.stop(Cut::Cancelled));
        }
        if Instant::now() >= self.guard.deadline {
            return Err(self.stop(Cut::Deadline));
        }
        let output = self.output.get();
        if output - self.mark_output.get() >= self.limits.stall_output {
            self.mark_output.set(output);
            self.mark_input.set(self.input.get());
        } else if self.input.get() - self.mark_input.get() > self.limits.stall_input {
            return Err(self.stop(Cut::Stalled));
        }
        Ok(())
    }
}

struct Counted<R> {
    inner: R,
    watch: Rc<Watch>,
}

impl<R: Read> Read for Counted<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.watch.poll()?;
        match self.inner.read(buf) {
            Ok(read) => {
                let input = self.watch.input.get().saturating_add(read as u64);
                self.watch.input.set(input);
                Ok(read)
            }
            Err(error) => {
                match error.kind() {
                    io::ErrorKind::TimedOut => self.watch.set_cut(Cut::Deadline),
                    _ => self.watch.source_failed.set(true),
                }
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
                    self.watch.poll()?;
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
                    self.watch.poll()?;
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
                        self.watch.poll()?;
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
}

impl<R: Read> Read for Bounded<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.watch.cut().is_some_and(|cut| cut == Cut::Output) {
            return Ok(0);
        }
        self.watch.poll()?;
        let room = self.limits.max_output - self.output;
        let ask =
            usize::try_from(room.saturating_add(1)).map_or(buf.len(), |ask| ask.min(buf.len()));
        let read = self.inner.read(&mut buf[..ask])?;
        if read as u64 > room {
            self.output = self.limits.max_output;
            self.watch.output.set(self.output);
            self.watch.set_cut(Cut::Output);
            return Ok(usize::try_from(room).unwrap_or(0));
        }
        self.output += read as u64;
        self.watch.output.set(self.output);
        Ok(read)
    }
}

pub fn watch(guard: Guard, limits: Limits) -> Rc<Watch> {
    Rc::new(Watch::new(guard, limits))
}

pub fn open<R: Read>(source: R, limits: Limits, watch: Rc<Watch>) -> io::Result<Bounded<R>> {
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
    Ok(Bounded {
        inner,
        watch,
        limits,
        output: 0,
    })
}
