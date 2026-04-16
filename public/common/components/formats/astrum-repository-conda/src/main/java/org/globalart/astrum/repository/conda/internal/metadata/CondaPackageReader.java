package org.globalart.astrum.repository.conda.internal.metadata;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import org.globalart.astrum.repository.conda.AssetKind;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorInputStream;
import org.apache.commons.compress.compressors.zstandard.ZstdCompressorInputStream;

public final class CondaPackageReader
{
  private static final String INDEX_JSON = "info/index.json";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final TypeReference<LinkedHashMap<String, Object>> MAP_TYPE = new TypeReference<>() { };

  private CondaPackageReader() {
  }

  public static Map<String, Object> readIndex(final InputStream in, final AssetKind assetKind) throws IOException {
    byte[] index = assetKind == AssetKind.ARCH_CONDA_PACKAGE ? readFromConda(in) : readFromTarBz2(in);
    if (index == null) {
      throw new InvalidCondaPackageException("Package does not contain " + INDEX_JSON);
    }
    try {
      return MAPPER.readValue(index, MAP_TYPE);
    }
    catch (IOException e) {
      throw new InvalidCondaPackageException("Unable to parse " + INDEX_JSON + ": " + e.getMessage());
    }
  }

  private static byte[] readFromTarBz2(final InputStream in) throws IOException {
    try (BZip2CompressorInputStream bzip = new BZip2CompressorInputStream(new BufferedInputStream(in))) {
      return findIndex(bzip);
    }
    catch (IOException e) {
      throw new InvalidCondaPackageException("Unable to read .tar.bz2 package: " + e.getMessage());
    }
  }

  private static byte[] readFromConda(final InputStream in) throws IOException {
    try (ZipInputStream zip = new ZipInputStream(new BufferedInputStream(in))) {
      ZipEntry entry;
      while ((entry = zip.getNextEntry()) != null) {
        String name = entry.getName();
        if (name.startsWith("info-") && name.endsWith(".tar.zst")) {
          try (ZstdCompressorInputStream zstd = new ZstdCompressorInputStream(zip)) {
            return findIndex(zstd);
          }
        }
      }
      return null;
    }
    catch (IOException e) {
      throw new InvalidCondaPackageException("Unable to read .conda package: " + e.getMessage());
    }
  }

  private static byte[] findIndex(final InputStream tarStream) throws IOException {
    TarArchiveInputStream tar = new TarArchiveInputStream(tarStream);
    TarArchiveEntry entry;
    while ((entry = tar.getNextEntry()) != null) {
      String name = entry.getName();
      if (name.startsWith("./")) {
        name = name.substring(2);
      }
      if (INDEX_JSON.equals(name) && entry.isFile()) {
        return tar.readAllBytes();
      }
    }
    return null;
  }
}
