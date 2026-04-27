
package org.globalart.astrum.repository.r.internal.util;

import java.math.BigInteger;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;

import static com.google.common.base.Preconditions.checkNotNull;

public class RPackageVersion
    implements Comparable<RPackageVersion>
{
  private final String originalVersion;

  private final List<String> parts;

  public RPackageVersion(final String version) {
    this.originalVersion = checkNotNull(version).trim();
    this.parts = Arrays.asList(originalVersion.split("[.-]", -1));
  }

  @Override
  public int compareTo(final RPackageVersion other) {
    int common = Math.min(parts.size(), other.parts.size());
    for (int i = 0; i < common; i++) {
      int result = comparePart(parts.get(i), other.parts.get(i));
      if (result != 0) {
        return result;
      }
    }
    return Integer.compare(parts.size(), other.parts.size());
  }

  private static int comparePart(final String left, final String right) {
    boolean leftNumeric = isNumeric(left);
    boolean rightNumeric = isNumeric(right);
    if (leftNumeric && rightNumeric) {
      return new BigInteger(left).compareTo(new BigInteger(right));
    }
    if (leftNumeric != rightNumeric) {
      return leftNumeric ? 1 : -1;
    }
    return left.compareTo(right);
  }

  private static boolean isNumeric(final String part) {
    return !part.isEmpty() && part.chars().allMatch(Character::isDigit);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    return compareTo((RPackageVersion) o) == 0;
  }

  @Override
  public int hashCode() {
    return Objects.hash(parts.stream().map(part -> isNumeric(part) ? new BigInteger(part).toString() : part).toList());
  }

  @Override
  public String toString() {
    return originalVersion;
  }
}
