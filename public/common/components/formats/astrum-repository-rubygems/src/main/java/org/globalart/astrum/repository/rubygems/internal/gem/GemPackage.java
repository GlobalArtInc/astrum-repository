package org.globalart.astrum.repository.rubygems.internal.gem;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.zip.GZIPInputStream;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;

public final class GemPackage
{
  public static final String METADATA = "metadata.gz";

  public static final String DATA = "data.tar.gz";

  private static final int MAX_METADATA_SIZE = 32 * 1024 * 1024;

  private GemPackage() {
  }

  public static GemSpec readSpec(final InputStream gem) throws IOException {
    GemSpec spec = null;
    boolean data = false;
    TarArchiveInputStream tar = new TarArchiveInputStream(gem);
    TarArchiveEntry entry;
    try {
      while ((entry = tar.getNextEntry()) != null) {
        if (METADATA.equals(entry.getName()) && spec == null) {
          spec = GemSpecParser.parse(new ByteArrayInputStream(readMetadata(tar)));
        }
        else if (DATA.equals(entry.getName())) {
          data = true;
        }
        if (spec != null && data) {
          break;
        }
      }
    }
    catch (IllegalArgumentException e) {
      throw e;
    }
    catch (RuntimeException e) {
      throw new IOException("Not a gem file: " + e.getMessage(), e);
    }
    if (spec == null) {
      throw new IllegalArgumentException("Not a gem file: " + METADATA + " is missing");
    }
    if (!data) {
      throw new IllegalArgumentException("Not a gem file: " + DATA + " is missing");
    }
    return spec;
  }

  private static byte[] readMetadata(final InputStream entry) throws IOException {
    InputStream in = new GZIPInputStream(nonClosing(entry));
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    byte[] buffer = new byte[8192];
    int read;
    while ((read = in.read(buffer)) != -1) {
      if (out.size() + read > MAX_METADATA_SIZE) {
        throw new IllegalArgumentException("Gem specification is larger than " + MAX_METADATA_SIZE + " bytes");
      }
      out.write(buffer, 0, read);
    }
    return out.toByteArray();
  }

  private static InputStream nonClosing(final InputStream in) {
    return new InputStream()
    {
      @Override
      public int read() throws IOException {
        return in.read();
      }

      @Override
      public int read(final byte[] b, final int off, final int len) throws IOException {
        return in.read(b, off, len);
      }

      @Override
      public void close() {
      }
    };
  }
}
