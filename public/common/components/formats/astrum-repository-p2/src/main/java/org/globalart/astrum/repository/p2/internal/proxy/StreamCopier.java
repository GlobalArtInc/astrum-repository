
package org.globalart.astrum.repository.p2.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;
import java.util.zip.ZipOutputStream;

import org.apache.commons.compress.compressors.xz.XZCompressorInputStream;
import org.apache.commons.compress.compressors.xz.XZCompressorOutputStream;

@FunctionalInterface
public interface StreamCopier
{
  void process(StreamTransformer transformer) throws IOException;

  @FunctionalInterface
  public interface StreamTransformer
  {
    void transform(InputStream in, OutputStream out) throws IOException;
  }

  static StreamCopier zip(final String filename, final InputStream in, final OutputStream out) {
    return transformer -> {
      try (ZipInputStream zipIn = new ZipInputStream(in); ZipOutputStream zipOut = new ZipOutputStream(out)) {
        ZipEntry entry;
        while ((entry = zipIn.getNextEntry()) != null) {
          if (!entry.isDirectory() && filename.equals(entry.getName())) {
            zipOut.putNextEntry(new ZipEntry(filename));
            transformer.transform(zipIn, zipOut);
            return;
          }
        }
        throw new IOException("Unable to locate entry with filename: " + filename);
      }
    };
  }

  static StreamCopier xz(final InputStream in, final OutputStream out) {
    return transformer -> {
      try (XZCompressorInputStream xzIn = new XZCompressorInputStream(in);
          XZCompressorOutputStream xzOut = new XZCompressorOutputStream(out)) {
        transformer.transform(xzIn, xzOut);
      }
    };
  }

  static StreamCopier passthrough(final InputStream in, final OutputStream out) {
    return transformer -> transformer.transform(in, out);
  }

  static StreamCopier copierFor(
      final String path,
      final String filename,
      final InputStream in,
      final OutputStream out) throws IOException
  {
    if (path.endsWith(".jar")) {
      return StreamCopier.zip(filename, in, out);
    }
    if (path.endsWith(".xml.xz")) {
      return StreamCopier.xz(in, out);
    }
    if (path.endsWith(".xml")) {
      return StreamCopier.passthrough(in, out);
    }
    throw new IOException("Unknown metadata file " + path + " for " + filename);
  }
}
