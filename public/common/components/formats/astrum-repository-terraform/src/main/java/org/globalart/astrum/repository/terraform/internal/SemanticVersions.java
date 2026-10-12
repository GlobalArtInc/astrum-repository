package org.globalart.astrum.repository.terraform.internal;

import java.util.Comparator;

/**
 * Orders versions by Semantic Versioning precedence; build metadata does not count.
 */
public final class SemanticVersions
{
  public static final Comparator<String> ORDER = SemanticVersions::compare;

  private SemanticVersions() {
  }

  public static int compare(final String left, final String right) {
    String a = stripBuild(left);
    String b = stripBuild(right);
    int dashA = a.indexOf('-');
    int dashB = b.indexOf('-');
    String coreA = dashA < 0 ? a : a.substring(0, dashA);
    String coreB = dashB < 0 ? b : b.substring(0, dashB);
    int core = compareIdentifiers(coreA.split("\\."), coreB.split("\\."), true);
    if (core != 0) {
      return core;
    }
    if (dashA < 0 || dashB < 0) {
      return Boolean.compare(dashA < 0, dashB < 0);
    }
    return compareIdentifiers(a.substring(dashA + 1).split("\\."), b.substring(dashB + 1).split("\\."), false);
  }

  private static int compareIdentifiers(final String[] a, final String[] b, final boolean numericOnly) {
    for (int i = 0; i < Math.min(a.length, b.length); i++) {
      int result = compareIdentifier(a[i], b[i], numericOnly);
      if (result != 0) {
        return result;
      }
    }
    return Integer.compare(a.length, b.length);
  }

  private static int compareIdentifier(final String a, final String b, final boolean numericOnly) {
    boolean numericA = isNumeric(a);
    boolean numericB = isNumeric(b);
    if (numericA && numericB) {
      String trimmedA = a.replaceFirst("^0+(?=.)", "");
      String trimmedB = b.replaceFirst("^0+(?=.)", "");
      int length = Integer.compare(trimmedA.length(), trimmedB.length());
      return length != 0 ? length : trimmedA.compareTo(trimmedB);
    }
    if (numericOnly || (!numericA && !numericB)) {
      return a.compareTo(b);
    }
    return numericA ? -1 : 1;
  }

  private static boolean isNumeric(final String value) {
    return !value.isEmpty() && value.chars().allMatch(Character::isDigit);
  }

  private static String stripBuild(final String version) {
    int plus = version.indexOf('+');
    return plus < 0 ? version : version.substring(0, plus);
  }
}
