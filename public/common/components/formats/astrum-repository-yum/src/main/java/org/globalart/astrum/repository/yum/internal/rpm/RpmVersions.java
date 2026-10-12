package org.globalart.astrum.repository.yum.internal.rpm;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class RpmVersions
{
  private static final Pattern GLIBC_VERSION = Pattern.compile("\\(GLIBC_([^)]*)\\)");

  private RpmVersions() {
  }

  public static int compare(final String first, final String second) {
    if (first.equals(second)) {
      return 0;
    }
    int i = 0;
    int j = 0;
    while (i < first.length() || j < second.length()) {
      while (i < first.length() && isSeparator(first.charAt(i))) {
        i++;
      }
      while (j < second.length() && isSeparator(second.charAt(j))) {
        j++;
      }
      if (at(first, i, '~') || at(second, j, '~')) {
        if (!at(first, i, '~')) {
          return 1;
        }
        if (!at(second, j, '~')) {
          return -1;
        }
        i++;
        j++;
        continue;
      }
      if (at(first, i, '^') || at(second, j, '^')) {
        if (i >= first.length()) {
          return -1;
        }
        if (j >= second.length()) {
          return 1;
        }
        if (!at(first, i, '^')) {
          return 1;
        }
        if (!at(second, j, '^')) {
          return -1;
        }
        i++;
        j++;
        continue;
      }
      if (i >= first.length() || j >= second.length()) {
        break;
      }
      int startFirst = i;
      int startSecond = j;
      boolean numeric = Character.isDigit(first.charAt(i));
      if (numeric) {
        while (i < first.length() && Character.isDigit(first.charAt(i))) {
          i++;
        }
        while (j < second.length() && Character.isDigit(second.charAt(j))) {
          j++;
        }
      }
      else {
        while (i < first.length() && Character.isLetter(first.charAt(i))) {
          i++;
        }
        while (j < second.length() && Character.isLetter(second.charAt(j))) {
          j++;
        }
      }
      String segmentFirst = first.substring(startFirst, i);
      String segmentSecond = second.substring(startSecond, j);
      if (segmentSecond.isEmpty()) {
        return numeric ? 1 : -1;
      }
      int result;
      if (numeric) {
        String a = stripZeros(segmentFirst);
        String b = stripZeros(segmentSecond);
        result = a.length() != b.length() ? Integer.compare(a.length(), b.length()) : a.compareTo(b);
      }
      else {
        result = segmentFirst.compareTo(segmentSecond);
      }
      if (result != 0) {
        return result < 0 ? -1 : 1;
      }
    }
    if (i >= first.length() && j >= second.length()) {
      return 0;
    }
    return i >= first.length() ? -1 : 1;
  }

  static int compareLibcRequires(final String first, final String second) {
    Matcher a = GLIBC_VERSION.matcher(first);
    Matcher b = GLIBC_VERSION.matcher(second);
    boolean hasFirst = a.find();
    boolean hasSecond = b.find();
    if (!hasFirst || !hasSecond) {
      return Boolean.compare(hasFirst, hasSecond);
    }
    return compare(a.group(1), b.group(1));
  }

  private static boolean at(final String value, final int index, final char c) {
    return index < value.length() && value.charAt(index) == c;
  }

  private static boolean isSeparator(final char c) {
    return !Character.isLetterOrDigit(c) && c != '~' && c != '^';
  }

  private static String stripZeros(final String number) {
    int index = 0;
    while (index < number.length() - 1 && number.charAt(index) == '0') {
      index++;
    }
    return number.substring(index);
  }
}
