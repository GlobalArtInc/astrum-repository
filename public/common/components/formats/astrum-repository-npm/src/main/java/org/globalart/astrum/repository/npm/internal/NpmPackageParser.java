
package org.globalart.astrum.repository.npm.internal;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.stereotype.Component;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Map;
import java.util.function.Supplier;



import com.google.common.annotations.VisibleForTesting;
import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.ArchiveInputStream;
import org.apache.commons.compress.archivers.ArchiveStreamFactory;
import org.apache.commons.compress.compressors.CompressorStreamFactory;

import static java.util.Collections.emptyMap;
import static org.apache.commons.compress.archivers.ArchiveStreamFactory.TAR;
import static org.apache.commons.compress.compressors.CompressorStreamFactory.GZIP;

/**
 * Parser for npm packages that will open up the tarball, extract the package.json if present, and return a map with the
 * attributes parsed from the npm package.
 *
 * @since 3.7
 */
@Component
public class NpmPackageParser
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String SEPARATOR = "/";

  private static final String PACKAGE_JSON_SUBPATH = SEPARATOR + "package.json";

  /**
   * Parses the package.json in the supplied tar.gz if present and extractable. In all other situations, an empty map
   * will be returned indicating the absence of (or inability to extract) a valid package.json file and its contents.
   */
  public Map<String, Object> parsePackageJson(final Supplier<InputStream> supplier) {
    try (InputStream is = new BufferedInputStream(supplier.get())) {
      final CompressorStreamFactory compressorStreamFactory = new CompressorStreamFactory();
      try (InputStream cis = compressorStreamFactory.createCompressorInputStream(GZIP, is)) {
        final ArchiveStreamFactory archiveFactory = new ArchiveStreamFactory();
        try (ArchiveInputStream ais = archiveFactory.createArchiveInputStream(TAR, cis)) {
          return parsePackageJsonInternal(ais);
        }
      }
    }
    catch (Exception e) {
      log.debug("Error occurred while processing package.json, returning empty map to continue", e);
      return emptyMap();
    }
  }

  /**
   * Performs the actual parsing of the package.json file if it exists.
   */
  private Map<String, Object> parsePackageJsonInternal(final ArchiveInputStream archiveInputStream)
      throws IOException
  {
    ArchiveEntry entry = archiveInputStream.getNextEntry();
    while (entry != null) {
      if (isPackageJson(entry)) {
        return NpmJsonUtils.parse(() -> archiveInputStream).backing();
      }
      entry = archiveInputStream.getNextEntry();
    }
    return emptyMap();
  }

  /**
   * Determines if the specified archive entry's name could represent a valid package.json file. Typically these would
   * be under {@code package/package.json}, but there are tarballs out there that have the package.json in a different
   * directory.
   */
  @VisibleForTesting
  boolean isPackageJson(final ArchiveEntry entry) {
    if (entry.isDirectory()) {
      return false;
    }
    String name = entry.getName();
    if (name == null) {
      return false;
    }
    else if (!name.endsWith(PACKAGE_JSON_SUBPATH)) {
      return false; // not a package.json file
    }
    else if (name.startsWith(PACKAGE_JSON_SUBPATH)) {
      return false; // should not be at the root, should be under the path containing the actual package, whatever it is
    }
    else {
      return name.indexOf(PACKAGE_JSON_SUBPATH) == name.indexOf(SEPARATOR); // path should only be one directory deep
    }
  }
}
