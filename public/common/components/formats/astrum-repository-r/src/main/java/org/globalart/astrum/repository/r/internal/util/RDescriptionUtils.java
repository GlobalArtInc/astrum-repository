
package org.globalart.astrum.repository.r.internal.util;

import java.io.IOException;
import java.io.InputStream;
import java.util.Locale;
import java.util.Map;
import java.util.regex.Pattern;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.InvalidContentException;

import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.ArchiveInputStream;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.r.internal.util.RMetadataUtils.parseDescriptionFile;

public final class RDescriptionUtils
{
  private static final Pattern DESCRIPTION_FILE_PATTERN = Pattern.compile("^(\\./)?[^/]+/DESCRIPTION$");

  private static final int MAX_DESCRIPTION_SIZE = 1024 * 1024;

  public static Map<String, String> extractDescriptionFromArchive(
      final String filename,
      final InputStreamSupplier inputStreamSupplier)
  {
    checkNotNull(filename);
    checkNotNull(inputStreamSupplier);
    String lowerCaseFilename = filename.toLowerCase(Locale.ENGLISH);
    try (InputStream is = checkNotNull(inputStreamSupplier.get())) {
      if (lowerCaseFilename.endsWith(".tar.gz") || lowerCaseFilename.endsWith(".tgz")) {
        try (GzipCompressorInputStream gzip = new GzipCompressorInputStream(is, true);
             TarArchiveInputStream tar = new TarArchiveInputStream(gzip)) {
          return extractFromArchive(filename, tar);
        }
      }
      if (lowerCaseFilename.endsWith(".zip")) {
        try (ZipArchiveInputStream zip = new ZipArchiveInputStream(is)) {
          return extractFromArchive(filename, zip);
        }
      }
    }
    catch (IOException | RuntimeException e) {
      if (e instanceof InvalidContentException invalid) {
        throw invalid;
      }
      throw new InvalidContentException("Unable to read R package " + filename + ": " + e.getMessage());
    }
    throw new InvalidContentException("Unexpected file extension for file: " + filename);
  }

  private static Map<String, String> extractFromArchive(
      final String filename,
      final ArchiveInputStream<? extends ArchiveEntry> archive) throws IOException
  {
    ArchiveEntry entry = archive.getNextEntry();
    while (entry != null) {
      if (!entry.isDirectory() && DESCRIPTION_FILE_PATTERN.matcher(entry.getName()).matches()) {
        byte[] bytes = archive.readNBytes(MAX_DESCRIPTION_SIZE + 1);
        if (bytes.length > MAX_DESCRIPTION_SIZE) {
          throw new InvalidContentException("DESCRIPTION of " + filename + " is too large");
        }
        return parseDescriptionFile(bytes);
      }
      entry = archive.getNextEntry();
    }
    throw new InvalidContentException("No DESCRIPTION file found in " + filename);
  }

  private RDescriptionUtils() {
  }
}
