package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.ByteArrayOutputStream;
import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.io.PushbackInputStream;
import java.util.zip.CRC32;
import java.util.zip.DataFormatException;
import java.util.zip.Deflater;
import java.util.zip.GZIPOutputStream;
import java.util.zip.Inflater;

public final class GzipMembers
{
  private static final int BUFFER_SIZE = 8192;

  private static final int FHCRC = 2;

  private static final int FEXTRA = 4;

  private static final int FNAME = 8;

  private static final int FCOMMENT = 16;

  private final PushbackInputStream in;

  private final long maxInflatedSize;

  public GzipMembers(final InputStream in, final long maxInflatedSize) {
    this.in = new PushbackInputStream(in, BUFFER_SIZE);
    this.maxInflatedSize = maxInflatedSize;
  }

  public Member next() throws IOException {
    int first = in.read();
    if (first < 0) {
      return null;
    }
    in.unread(first);
    RawBuffer raw = new RawBuffer();
    ByteArrayOutputStream inflated = new ByteArrayOutputStream();
    readHeader(raw);
    inflate(raw, inflated);
    byte[] trailer = readFully(8);
    raw.write(trailer);
    byte[] data = inflated.toByteArray();
    CRC32 crc = new CRC32();
    crc.update(data);
    long expectedCrc = littleEndian(trailer, 0);
    long expectedSize = littleEndian(trailer, 4);
    if (crc.getValue() != expectedCrc || (data.length & 0xffffffffL) != expectedSize) {
      throw new IOException("Corrupt gzip member");
    }
    return new Member(raw.toByteArray(), data);
  }

  private void readHeader(final RawBuffer raw) throws IOException {
    byte[] header = readFully(10);
    raw.write(header);
    if ((header[0] & 0xff) != 0x1f || (header[1] & 0xff) != 0x8b || header[2] != 8) {
      throw new IOException("Not a gzip stream");
    }
    int flags = header[3] & 0xff;
    if ((flags & FEXTRA) != 0) {
      byte[] length = readFully(2);
      raw.write(length);
      raw.write(readFully((length[0] & 0xff) | ((length[1] & 0xff) << 8)));
    }
    if ((flags & FNAME) != 0) {
      readZeroTerminated(raw);
    }
    if ((flags & FCOMMENT) != 0) {
      readZeroTerminated(raw);
    }
    if ((flags & FHCRC) != 0) {
      raw.write(readFully(2));
    }
  }

  private void inflate(final RawBuffer raw, final ByteArrayOutputStream inflated) throws IOException {
    Inflater inflater = new Inflater(true);
    try {
      byte[] input = new byte[BUFFER_SIZE];
      byte[] output = new byte[BUFFER_SIZE];
      int lastRead = 0;
      while (!inflater.finished()) {
        if (inflater.needsInput()) {
          lastRead = in.read(input);
          if (lastRead < 0) {
            throw new EOFException("Truncated gzip member");
          }
          raw.write(input, 0, lastRead);
          inflater.setInput(input, 0, lastRead);
        }
        int count = inflater.inflate(output);
        if (count == 0 && inflater.needsDictionary()) {
          throw new IOException("Unsupported gzip member");
        }
        inflated.write(output, 0, count);
        if (inflated.size() > maxInflatedSize) {
          throw new IOException("Gzip member exceeds " + maxInflatedSize + " bytes");
        }
      }
      int remaining = inflater.getRemaining();
      if (remaining > 0) {
        in.unread(input, lastRead - remaining, remaining);
        raw.truncate(remaining);
      }
    }
    catch (DataFormatException e) {
      throw new IOException("Corrupt gzip member", e);
    }
    finally {
      inflater.end();
    }
  }

  private void readZeroTerminated(final RawBuffer raw) throws IOException {
    int b;
    do {
      b = in.read();
      if (b < 0) {
        throw new EOFException("Truncated gzip header");
      }
      raw.write(b);
    }
    while (b != 0);
  }

  private byte[] readFully(final int length) throws IOException {
    byte[] bytes = new byte[length];
    int offset = 0;
    while (offset < length) {
      int read = in.read(bytes, offset, length - offset);
      if (read < 0) {
        throw new EOFException("Truncated gzip stream");
      }
      offset += read;
    }
    return bytes;
  }

  private static long littleEndian(final byte[] bytes, final int offset) {
    return (bytes[offset] & 0xffL)
        | ((bytes[offset + 1] & 0xffL) << 8)
        | ((bytes[offset + 2] & 0xffL) << 16)
        | ((bytes[offset + 3] & 0xffL) << 24);
  }

  public static byte[] gzip(final byte[] data) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (GZIPOutputStream gzip = new GZIPOutputStream(out)
    {
      {
        def.setLevel(Deflater.BEST_COMPRESSION);
      }
    })
    {
      gzip.write(data);
    }
    return out.toByteArray();
  }

  public record Member(byte[] raw, byte[] inflated)
  {
  }

  private static final class RawBuffer
      extends ByteArrayOutputStream
  {
    void truncate(final int length) {
      count -= length;
    }
  }
}
