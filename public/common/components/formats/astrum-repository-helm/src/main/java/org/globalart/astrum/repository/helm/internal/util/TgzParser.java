
package org.globalart.astrum.repository.helm.internal.util;

import org.springframework.stereotype.Component;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;


import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;

/**
 * Utility methods for working with tgz files
 *
 * @since 3.28
 */
@Component
public class TgzParser
{
  private static final String CHART_NAME = "Chart.yaml";

  public InputStream getChartFromInputStream(final InputStream is) throws IOException {
    try (GzipCompressorInputStream gzis = new GzipCompressorInputStream(is)) {
      try (TarArchiveInputStream tais = new TarArchiveInputStream(gzis)) {
        ArchiveEntry currentEntry;
        while ((currentEntry = tais.getNextEntry()) != null) {
          if (currentEntry.getName().endsWith(CHART_NAME)) {
            byte[] buf = new byte[(int) currentEntry.getSize()];
            tais.read(buf, 0, buf.length);
            return new ByteArrayInputStream(buf);
          }
        }
      }
    }
    throw new IllegalArgumentException(String.format("%s not found", CHART_NAME));
  }
}
