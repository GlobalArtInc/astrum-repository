package org.globalart.astrum.repository.nuget.internal.metadata;

import java.util.Locale;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public final class NugetVersion
    implements Comparable<NugetVersion>
{
  private static final Pattern PATTERN = Pattern.compile(
      "^(\\d+)(?:\\.(\\d+))?(?:\\.(\\d+))?(?:\\.(\\d+))?(?:-([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?(?:\\+([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?$");

  private final long major;

  private final long minor;

  private final long patch;

  private final long revision;

  @Nullable
  private final String release;

  @Nullable
  private final String metadata;

  private NugetVersion(
      final long major,
      final long minor,
      final long patch,
      final long revision,
      @Nullable final String release,
      @Nullable final String metadata)
  {
    this.major = major;
    this.minor = minor;
    this.patch = patch;
    this.revision = revision;
    this.release = release;
    this.metadata = metadata;
  }

  public static Optional<NugetVersion> parse(@Nullable final String value) {
    if (value == null) {
      return Optional.empty();
    }
    Matcher matcher = PATTERN.matcher(value.trim());
    if (!matcher.matches()) {
      return Optional.empty();
    }
    try {
      return Optional.of(new NugetVersion(
          number(matcher.group(1)),
          number(matcher.group(2)),
          number(matcher.group(3)),
          number(matcher.group(4)),
          matcher.group(5),
          matcher.group(6)));
    }
    catch (NumberFormatException e) {
      return Optional.empty();
    }
  }

  public static Optional<String> normalize(@Nullable final String value) {
    return parse(value).map(NugetVersion::toNormalizedString);
  }

  private static long number(@Nullable final String value) {
    return value == null ? 0 : Long.parseLong(value);
  }

  public boolean isPrerelease() {
    return release != null;
  }

  public boolean isSemVer2() {
    return (release != null && release.contains(".")) || metadata != null;
  }

  public String toNormalizedString() {
    StringBuilder builder = new StringBuilder().append(major).append('.').append(minor).append('.').append(patch);
    if (revision != 0) {
      builder.append('.').append(revision);
    }
    if (release != null) {
      builder.append('-').append(release);
    }
    return builder.toString();
  }

  public String toFullString() {
    return metadata == null ? toNormalizedString() : toNormalizedString() + '+' + metadata;
  }

  public String toLowerString() {
    return toNormalizedString().toLowerCase(Locale.ROOT);
  }

  @Override
  public int compareTo(final NugetVersion other) {
    int result = Long.compare(major, other.major);
    if (result == 0) {
      result = Long.compare(minor, other.minor);
    }
    if (result == 0) {
      result = Long.compare(patch, other.patch);
    }
    if (result == 0) {
      result = Long.compare(revision, other.revision);
    }
    if (result == 0) {
      result = compareRelease(release, other.release);
    }
    return result;
  }

  private static int compareRelease(@Nullable final String left, @Nullable final String right) {
    if (left == null && right == null) {
      return 0;
    }
    if (left == null) {
      return 1;
    }
    if (right == null) {
      return -1;
    }
    String[] leftParts = left.split("\\.");
    String[] rightParts = right.split("\\.");
    for (int i = 0; i < Math.min(leftParts.length, rightParts.length); i++) {
      int result = compareLabel(leftParts[i], rightParts[i]);
      if (result != 0) {
        return result;
      }
    }
    return Integer.compare(leftParts.length, rightParts.length);
  }

  private static int compareLabel(final String left, final String right) {
    boolean leftNumeric = left.chars().allMatch(Character::isDigit);
    boolean rightNumeric = right.chars().allMatch(Character::isDigit);
    if (leftNumeric && rightNumeric) {
      int lengthDifference = Integer.compare(stripZeros(left).length(), stripZeros(right).length());
      return lengthDifference != 0 ? lengthDifference : stripZeros(left).compareTo(stripZeros(right));
    }
    if (leftNumeric) {
      return -1;
    }
    if (rightNumeric) {
      return 1;
    }
    return left.compareToIgnoreCase(right);
  }

  private static String stripZeros(final String value) {
    int index = 0;
    while (index < value.length() - 1 && value.charAt(index) == '0') {
      index++;
    }
    return value.substring(index);
  }

  @Override
  public boolean equals(final Object other) {
    return other instanceof NugetVersion version && compareTo(version) == 0;
  }

  @Override
  public int hashCode() {
    return toLowerString().hashCode();
  }

  @Override
  public String toString() {
    return toFullString();
  }
}
