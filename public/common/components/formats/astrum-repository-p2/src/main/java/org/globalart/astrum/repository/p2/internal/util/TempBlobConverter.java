
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.jar.JarOutputStream;
import java.util.zip.GZIPInputStream;

import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.apache.commons.compress.java.util.jar.Pack200;
import org.apache.commons.io.input.BoundedInputStream;
import org.springframework.stereotype.Component;

@Component
public class TempBlobConverter
{
  public InputStream getJarFromPackGz(final TempBlob tempBlob) throws IOException {
    Path tempFile = Files.createTempFile("pack-file", ".jar");
    try {
      try (InputStream in = BoundedInputStream.builder().setInputStream(new GZIPInputStream(tempBlob.get())).get();
           OutputStream out = Files.newOutputStream(tempFile);
           JarOutputStream jar = new JarOutputStream(out)) {
        Pack200.newUnpacker().unpack(in, jar);
      }
      return Files.newInputStream(tempFile, StandardOpenOption.DELETE_ON_CLOSE);
    }
    catch (IOException | RuntimeException e) {
      Files.deleteIfExists(tempFile);
      throw e;
    }
  }
}
