package org.globalart.astrum.repository.yum.internal.rpm;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public record RpmFileName(String name, String version, String release, String arch)
{
  private static final Pattern PATTERN = Pattern.compile("^(.+)-([^-]+)-([^-]+)\\.([^.-]+)\\.rpm$");

  public static Optional<RpmFileName> parse(final String fileName) {
    Matcher matcher = PATTERN.matcher(fileName);
    if (!matcher.matches()) {
      return Optional.empty();
    }
    return Optional.of(new RpmFileName(matcher.group(1), matcher.group(2), matcher.group(3), matcher.group(4)));
  }

  public String versionRelease() {
    return version + "-" + release;
  }
}
