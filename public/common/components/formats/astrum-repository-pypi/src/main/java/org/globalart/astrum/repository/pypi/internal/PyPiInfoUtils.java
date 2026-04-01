
package org.globalart.astrum.repository.pypi.internal;

import java.io.BufferedInputStream;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableMap;
import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.ArchiveInputStream;
import org.apache.commons.compress.archivers.ArchiveStreamFactory;
import org.apache.commons.compress.compressors.CompressorInputStream;
import org.apache.commons.compress.compressors.CompressorStreamFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_ARCHIVE_TYPE;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_CLASSIFIERS;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.DIST_INFO_SUFFIX;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.EGG_INFO_FILENAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.EGG_INFO_SUFFIX;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.METADATA_FILENAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiConstants.PKG_INFO_FILENAME;

public final class PyPiInfoUtils
{
  private static final Logger log = LoggerFactory.getLogger(PyPiInfoUtils.class);

  private static final Map<String, String> NAME_SUBSTITUTIONS = new ImmutableMap.Builder<String, String>()
      .put("classifier", P_CLASSIFIERS)
      .build();

  public static Map<String, String> extractMetadata(final InputStream is) {
    checkNotNull(is);
    InputStream in = is.markSupported() ? is : new BufferedInputStream(is);
    Map<String, Exception> exceptions = new HashMap<>();

    try {
      String compressionType = CompressorStreamFactory.detect(in);
      try (CompressorInputStream cis = CompressorStreamFactory.getSingleton()
          .createCompressorInputStream(compressionType, in)) {
        InputStream decompressedInput = new BufferedInputStream(cis);
        String archiveType = ArchiveStreamFactory.detect(decompressedInput);
        Map<String, String> metadata = extractMetadataFromArchive(archiveType, decompressedInput);
        metadata.put(P_ARCHIVE_TYPE, archiveType + '.' + compressionType);
        return metadata;
      }
    }
    catch (Exception e) {
      exceptions.put("Unable to decompress PyPI archive", e);
    }

    try {
      String archiveType = ArchiveStreamFactory.detect(in);
      Map<String, String> metadata = extractMetadataFromArchive(archiveType, in);
      metadata.put(P_ARCHIVE_TYPE, archiveType);
      return metadata;
    }
    catch (Exception e) {
      exceptions.put("Unable to extract PyPI archive", e);
    }

    for (Map.Entry<String, Exception> entry : exceptions.entrySet()) {
      log.debug(entry.getKey(), entry.getValue());
    }
    log.warn("Unable to extract PyPI metadata from archive");
    return new LinkedHashMap<>();
  }

  private static Map<String, String> extractMetadataFromArchive(final String archiveType, final InputStream is)
      throws Exception
  {
    checkNotNull(archiveType);
    checkNotNull(is);
    ArchiveStreamFactory archiveFactory = new ArchiveStreamFactory();
    try (ArchiveInputStream<?> ais = archiveFactory.createArchiveInputStream(archiveType, is)) {
      return processArchiveEntries(ais);
    }
  }

  private static Map<String, String> processArchiveEntries(final ArchiveInputStream<?> ais) throws Exception {
    checkNotNull(ais);
    ArchiveEntry entry = ais.getNextEntry();
    while (entry != null) {
      if (isMetadataFile(entry)) {
        Map<String, String> results = new LinkedHashMap<>();
        for (Entry<String, List<String>> attribute : parsePackageInfo(ais).entrySet()) {
          results.put(attribute.getKey(), String.join("\n", attribute.getValue()));
        }
        return results;
      }
      entry = ais.getNextEntry();
    }
    return new LinkedHashMap<>();
  }

  @VisibleForTesting
  static Map<String, List<String>> parsePackageInfo(final InputStream in) throws Exception {
    checkNotNull(in);
    String text = new String(in.readAllBytes(), UTF_8).replace("\r\n", "\n").replace('\r', '\n');
    LinkedHashMap<String, List<String>> results = new LinkedHashMap<>();

    String[] lines = text.split("\n", -1);
    int index = 0;
    String headerName = null;
    StringBuilder headerValue = null;
    for (; index < lines.length; index++) {
      String line = lines[index];
      if (line.isEmpty()) {
        index++;
        break;
      }
      if ((line.charAt(0) == ' ' || line.charAt(0) == '\t') && headerName != null) {
        headerValue.append('\n').append(line);
        continue;
      }
      int colon = line.indexOf(':');
      if (colon <= 0) {
        continue;
      }
      addHeader(results, headerName, headerValue);
      headerName = line.substring(0, colon).trim();
      headerValue = new StringBuilder(line.substring(colon + 1).stripLeading());
    }
    addHeader(results, headerName, headerValue);

    if (!results.containsKey(PyPiAttributes.P_DESCRIPTION) && index < lines.length) {
      String body = String.join("\n", Arrays.asList(lines).subList(index, lines.length)).trim();
      if (!body.isEmpty()) {
        List<String> description = new ArrayList<>();
        description.add(body.replaceAll("[ ]*\\n[ ]*", "\n") + "\n");
        results.put(PyPiAttributes.P_DESCRIPTION, description);
      }
    }

    return results;
  }

  private static void addHeader(
      final Map<String, List<String>> results,
      final String headerName,
      final StringBuilder headerValue)
  {
    if (headerName == null) {
      return;
    }
    String underscoreName = headerName.toLowerCase().replace('-', '_');
    String name = NAME_SUBSTITUTIONS.getOrDefault(underscoreName, underscoreName);
    String raw = headerValue.toString();
    String value = raw.isBlank() ? "" : raw.replaceAll("[ ]*\\n[ ]*", "\n");
    results.computeIfAbsent(name, key -> new ArrayList<>()).add(value);
  }

  private static boolean isMetadataFile(final ArchiveEntry entry) {
    checkNotNull(entry);
    if (entry.isDirectory()) {
      return false;
    }
    String[] parts = entry.getName().split("/");
    return isEggInfoPath(parts) || isPackageInfoFilePath(parts) || isMetadataFilePath(parts);
  }

  private static boolean isEggInfoPath(final String[] parts) {
    checkNotNull(parts);
    if (parts.length == 2 && EGG_INFO_FILENAME.equals(parts[1])) {
      return true;
    }
    if (parts.length == 1) {
      return EGG_INFO_FILENAME.equals(parts[0]) || parts[0].endsWith(EGG_INFO_SUFFIX);
    }
    return false;
  }

  private static boolean isMetadataFilePath(final String[] parts) {
    checkNotNull(parts);
    return parts.length == 2 && parts[0].endsWith(DIST_INFO_SUFFIX) && METADATA_FILENAME.equals(parts[1]);
  }

  private static boolean isPackageInfoFilePath(final String[] parts) {
    checkNotNull(parts);
    if (parts.length == 3 && parts[1].endsWith(EGG_INFO_SUFFIX) && PKG_INFO_FILENAME.equals(parts[2])) {
      return true;
    }
    if (parts.length == 2 && EGG_INFO_FILENAME.equals(parts[0]) && PKG_INFO_FILENAME.equals(parts[1])) {
      return true;
    }
    return parts.length == 2 && PKG_INFO_FILENAME.equals(parts[1]);
  }

  private PyPiInfoUtils() {
  }
}
