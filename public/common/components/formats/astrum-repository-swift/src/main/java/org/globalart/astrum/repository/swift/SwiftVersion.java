package org.globalart.astrum.repository.swift;

import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public final class SwiftVersion
    implements Comparable<SwiftVersion>
{
  private static final String NUMBER = "0|[1-9]\\d*";

  private static final String PRERELEASE_IDENTIFIER = "(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*)";

  private static final Pattern SEMVER = Pattern.compile(
      "(" + NUMBER + ")\\.(" + NUMBER + ")\\.(" + NUMBER + ")"
          + "(?:-(" + PRERELEASE_IDENTIFIER + "(?:\\." + PRERELEASE_IDENTIFIER + ")*))?"
          + "(?:\\+([0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?");

  public static final Comparator<String> PRECEDENCE = Comparator.comparing(
      (String value) -> parse(value).orElse(null), Comparator.nullsFirst(Comparator.naturalOrder()));

  private final String text;

  private final long major;

  private final long minor;

  private final long patch;

  private final List<String> prerelease;

  private SwiftVersion(
      final String text,
      final long major,
      final long minor,
      final long patch,
      final List<String> prerelease)
  {
    this.text = text;
    this.major = major;
    this.minor = minor;
    this.patch = patch;
    this.prerelease = prerelease;
  }

  public static boolean isValid(@Nullable final String value) {
    return parse(value).isPresent();
  }

  public static Optional<SwiftVersion> parse(@Nullable final String value) {
    if (value == null || value.length() > 128) {
      return Optional.empty();
    }
    Matcher matcher = SEMVER.matcher(value);
    if (!matcher.matches()) {
      return Optional.empty();
    }
    try {
      List<String> prerelease = matcher.group(4) == null ? List.of() : List.of(matcher.group(4).split("\\."));
      return Optional.of(new SwiftVersion(value, Long.parseLong(matcher.group(1)), Long.parseLong(matcher.group(2)),
          Long.parseLong(matcher.group(3)), prerelease));
    }
    catch (NumberFormatException e) {
      return Optional.empty();
    }
  }

  public boolean isPrerelease() {
    return !prerelease.isEmpty();
  }

  @Override
  public int compareTo(final SwiftVersion other) {
    int result = Long.compare(major, other.major);
    if (result == 0) {
      result = Long.compare(minor, other.minor);
    }
    if (result == 0) {
      result = Long.compare(patch, other.patch);
    }
    if (result != 0) {
      return result;
    }
    if (prerelease.isEmpty() || other.prerelease.isEmpty()) {
      return Boolean.compare(prerelease.isEmpty(), other.prerelease.isEmpty());
    }
    for (int i = 0; i < Math.min(prerelease.size(), other.prerelease.size()); i++) {
      result = compareIdentifiers(prerelease.get(i), other.prerelease.get(i));
      if (result != 0) {
        return result;
      }
    }
    return Integer.compare(prerelease.size(), other.prerelease.size());
  }

  private static int compareIdentifiers(final String left, final String right) {
    boolean leftNumeric = left.chars().allMatch(Character::isDigit);
    boolean rightNumeric = right.chars().allMatch(Character::isDigit);
    if (leftNumeric && rightNumeric) {
      int lengths = Integer.compare(left.length(), right.length());
      return lengths != 0 ? lengths : left.compareTo(right);
    }
    if (leftNumeric != rightNumeric) {
      return leftNumeric ? -1 : 1;
    }
    return left.compareTo(right);
  }

  @Override
  public boolean equals(final Object other) {
    return other instanceof SwiftVersion version && text.equals(version.text);
  }

  @Override
  public int hashCode() {
    return Objects.hash(text);
  }

  @Override
  public String toString() {
    return text;
  }
}
