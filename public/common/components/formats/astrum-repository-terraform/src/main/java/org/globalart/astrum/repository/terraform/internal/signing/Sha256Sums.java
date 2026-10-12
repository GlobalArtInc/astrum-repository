package org.globalart.astrum.repository.terraform.internal.signing;

import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * A checksums document in the format of {@code sha256sum}: one {@code <hex digest>  <file name>} line per file.
 */
public final class Sha256Sums
{
  private static final Pattern LINE = Pattern.compile("([0-9a-fA-F]{64})\\s+\\*?(\\S+)\\s*");

  private final Map<String, String> digests;

  private Sha256Sums(final Map<String, String> digests) {
    this.digests = digests;
  }

  public static Sha256Sums of(final Map<String, String> digests) {
    Map<String, String> sorted = new TreeMap<>();
    digests.forEach((file, digest) -> sorted.put(file, digest.toLowerCase(Locale.ROOT)));
    return new Sha256Sums(sorted);
  }

  /**
   * @throws IllegalArgumentException when a non-empty line is not a checksum line
   */
  public static Sha256Sums parse(final byte[] document) {
    Map<String, String> digests = new TreeMap<>();
    String[] lines = new String(document, UTF_8).split("\r?\n");
    for (int i = 0; i < lines.length; i++) {
      String line = lines[i];
      if (line.isBlank()) {
        continue;
      }
      Matcher matcher = LINE.matcher(line);
      if (!matcher.matches()) {
        throw new IllegalArgumentException("Line " + (i + 1) + " of the checksums is not '<sha256>  <file name>'");
      }
      digests.put(matcher.group(2), matcher.group(1).toLowerCase(Locale.ROOT));
    }
    if (digests.isEmpty()) {
      throw new IllegalArgumentException("The checksums document is empty");
    }
    return new Sha256Sums(digests);
  }

  public Optional<String> digest(final String fileName) {
    return Optional.ofNullable(digests.get(fileName));
  }

  public Map<String, String> digests() {
    return digests;
  }

  public byte[] toBytes() {
    StringBuilder document = new StringBuilder();
    digests.forEach((file, digest) -> document.append(digest).append("  ").append(file).append('\n'));
    return document.toString().getBytes(UTF_8);
  }
}
