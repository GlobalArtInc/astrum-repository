
package org.globalart.astrum.repository.apt.internal;

import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.apt.internal.debian.ControlFile;
import org.globalart.astrum.repository.apt.internal.debian.ControlFileParser;
import org.globalart.astrum.repository.apt.internal.debian.PackageInfo;

import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.ar.ArArchiveInputStream;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;
import org.apache.commons.compress.compressors.xz.XZCompressorInputStream;
import org.apache.commons.compress.compressors.zstandard.ZstdCompressorInputStream;
import org.apache.commons.io.input.CloseShieldInputStream;

/**
 * @since 3.17
 */
public class AptPackageParser
{
  private AptPackageParser() {
    throw new IllegalAccessError("Utility class");
  }

  public static PackageInfo parsePackageInfo(final InputStreamSupplier supplier) throws IOException {
    ControlFile controlFile = parsePackageInternal(supplier);
    if (controlFile == null) {
      throw new IOException("Invalid debian package: no control file");
    }
    return new PackageInfo(controlFile);
  }

  private static ControlFile parsePackageInternal(final InputStreamSupplier supplier) throws IOException {
    try (ArArchiveInputStream is = new ArArchiveInputStream(supplier.get())) {
      ControlFile control = null;
      ArchiveEntry debEntry;
      while ((debEntry = is.getNextEntry()) != null) {
        InputStream controlStream;
        switch (debEntry.getName()) {
          case "control.tar":
            controlStream = new CloseShieldInputStream(is);
            break;
          case "control.tar.gz":
            controlStream = new GzipCompressorInputStream(new CloseShieldInputStream(is));
            break;
          case "control.tar.xz":
            controlStream = new XZCompressorInputStream(new CloseShieldInputStream(is));
            break;
          case "control.tar.zst":
            controlStream = new ZstdCompressorInputStream(new CloseShieldInputStream(is));
            break;
          default:
            continue;
        }

        try (TarArchiveInputStream controlTarStream = new TarArchiveInputStream(controlStream)) {
          ArchiveEntry tarEntry;
          while ((tarEntry = controlTarStream.getNextEntry()) != null) {
            if ("control".equals(tarEntry.getName()) || "./control".equals(tarEntry.getName())) {
              control = new ControlFileParser().parseControlFile(controlTarStream);
            }
          }
        }
      }
      return control;
    }
  }
}
