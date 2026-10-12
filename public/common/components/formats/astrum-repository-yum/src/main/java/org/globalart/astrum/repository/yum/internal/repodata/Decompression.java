package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;

import org.apache.commons.compress.compressors.CompressorException;
import org.apache.commons.compress.compressors.CompressorStreamFactory;

public final class Decompression
{
  private Decompression() {
  }

  public static InputStream open(final InputStream raw) throws IOException {
    InputStream in = raw.markSupported() ? raw : new BufferedInputStream(raw, 65536);
    in.mark(16);
    int first = in.read();
    in.reset();
    if (first == '<' || first == 0xEF || first == ' ' || first == '\n' || first == '\r' || first == '\t') {
      return in;
    }
    try {
      return new CompressorStreamFactory(true).createCompressorInputStream(in);
    }
    catch (CompressorException e) {
      throw new IOException("Unsupported compression of repository metadata: " + e.getMessage(), e);
    }
  }
}
