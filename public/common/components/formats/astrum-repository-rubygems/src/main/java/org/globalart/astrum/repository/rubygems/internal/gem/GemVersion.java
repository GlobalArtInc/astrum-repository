package org.globalart.astrum.repository.rubygems.internal.gem;

import java.math.BigInteger;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class GemVersion
{
  public static final Comparator<String> COMPARATOR = GemVersion::compare;

  private static final Pattern VALID =
      Pattern.compile("[0-9]+(?>\\.[0-9a-zA-Z]+)*(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?");

  private static final Pattern SEGMENT = Pattern.compile("[0-9]+|[a-zA-Z]+");

  private static final Pattern LETTER = Pattern.compile("[a-zA-Z]");

  private static final int MAX_LENGTH = 128;

  private GemVersion() {
  }

  public static boolean isValid(final String version) {
    return version != null && version.length() <= MAX_LENGTH && VALID.matcher(version.strip()).matches();
  }

  public static boolean isPrerelease(final String version) {
    return version != null && LETTER.matcher(version).find();
  }

  public static int compare(final String left, final String right) {
    if (left.equals(right)) {
      return 0;
    }
    List<Object> lhs = canonicalSegments(left);
    List<Object> rhs = canonicalSegments(right);
    int limit = Math.max(lhs.size(), rhs.size());
    for (int i = 0; i < limit; i++) {
      Object l = i < lhs.size() ? lhs.get(i) : BigInteger.ZERO;
      Object r = i < rhs.size() ? rhs.get(i) : BigInteger.ZERO;
      if (l.equals(r)) {
        continue;
      }
      if (l instanceof String && r instanceof BigInteger) {
        return -1;
      }
      if (l instanceof BigInteger && r instanceof String) {
        return 1;
      }
      if (l instanceof BigInteger lNumber) {
        return Integer.signum(lNumber.compareTo((BigInteger) r));
      }
      return Integer.signum(((String) l).compareTo((String) r));
    }
    return 0;
  }

  static List<Object> segments(final String version) {
    List<Object> segments = new ArrayList<>();
    Matcher matcher = SEGMENT.matcher(version.strip().replace("-", ".pre."));
    while (matcher.find()) {
      String segment = matcher.group();
      segments.add(Character.isDigit(segment.charAt(0)) ? new BigInteger(segment) : segment);
    }
    return segments;
  }

  static List<Object> canonicalSegments(final String version) {
    List<Object> segments = segments(version);
    int stringStart = 0;
    while (stringStart < segments.size() && !(segments.get(stringStart) instanceof String)) {
      stringStart++;
    }
    List<Object> canonical = new ArrayList<>(dropTrailingZeros(segments.subList(0, stringStart)));
    canonical.addAll(dropTrailingZeros(segments.subList(stringStart, segments.size())));
    return canonical;
  }

  private static List<Object> dropTrailingZeros(final List<Object> segments) {
    int end = segments.size();
    while (end > 0 && BigInteger.ZERO.equals(segments.get(end - 1))) {
      end--;
    }
    return segments.subList(0, end);
  }
}
