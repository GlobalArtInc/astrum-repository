
package org.globalart.astrum.repository.r.internal.util;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.zip.GZIPInputStream;
import java.util.zip.GZIPOutputStream;

import javax.annotation.Nullable;

import static java.nio.charset.StandardCharsets.ISO_8859_1;

public final class RPackagesUtils
{
  private static final String PACKAGE_FIELD = "Package:";

  public static List<String> stanzas(final String packages) {
    List<String> stanzas = new ArrayList<>();
    StringBuilder stanza = new StringBuilder();
    for (String line : packages.replace("\r\n", "\n").split("\n", -1)) {
      if (line.isBlank()) {
        if (!stanza.isEmpty()) {
          stanzas.add(stanza.toString());
          stanza.setLength(0);
        }
      }
      else {
        stanza.append(line).append('\n');
      }
    }
    if (!stanza.isEmpty()) {
      stanzas.add(stanza.toString());
    }
    return stanzas;
  }

  @Nullable
  public static String packageName(final String stanza) {
    for (String line : stanza.split("\n")) {
      if (line.startsWith(PACKAGE_FIELD)) {
        return line.substring(PACKAGE_FIELD.length()).strip();
      }
    }
    return null;
  }

  public static MergeResult merge(final List<String> members) {
    Set<String> seen = new HashSet<>();
    StringBuilder merged = new StringBuilder();
    int count = 0;
    for (String member : members) {
      Set<String> names = new HashSet<>();
      for (String stanza : stanzas(member)) {
        String name = packageName(stanza);
        if (name == null || seen.contains(name)) {
          continue;
        }
        names.add(name);
        if (!merged.isEmpty()) {
          merged.append('\n');
        }
        merged.append(stanza);
        count++;
      }
      seen.addAll(names);
    }
    return new MergeResult(merged.toString(), count);
  }

  public static String readPackages(final InputStream in) throws IOException {
    byte[] bytes = in.readAllBytes();
    if (isGzip(bytes)) {
      try (InputStream gzip = new GZIPInputStream(new ByteArrayInputStream(bytes))) {
        bytes = gzip.readAllBytes();
      }
    }
    return new String(bytes, ISO_8859_1);
  }

  public static byte[] toBytes(final String packages, final boolean gzip) {
    byte[] bytes = packages.getBytes(ISO_8859_1);
    return gzip ? gzip(bytes) : bytes;
  }

  public static byte[] gzip(final byte[] bytes) {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (GZIPOutputStream gzip = new GZIPOutputStream(out)) {
      gzip.write(bytes);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    return out.toByteArray();
  }

  private static boolean isGzip(final byte[] bytes) {
    return bytes.length > 2 && (bytes[0] & 0xff) == 0x1f && (bytes[1] & 0xff) == 0x8b;
  }

  public record MergeResult(String packages, int packageCount)
  {
  }

  private RPackagesUtils() {
  }
}
