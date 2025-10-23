
package org.globalart.astrum.common.app;

import java.util.Comparator;
import java.util.regex.Pattern;

import org.eclipse.aether.util.version.GenericVersionScheme;
import org.eclipse.aether.version.InvalidVersionSpecificationException;
import org.eclipse.aether.version.Version;
import org.springframework.stereotype.Component;

/**
 * Implementation of {@link Comparator} for comparing {@link Version} Strings, specifically using a {@link
 * GenericVersionScheme}
 *
 * The non version like Strings are sorted first using String.compareTo followed by semantic version ordering.
 *
 * Example version-like Strings: 1.0, 1.0-beta4, 1.0-SNAPSHOT, 2.0.0
 * Example non version-like Strings: latest, release, beta5, 0xFF, ABCDEF
 *
 * @since 3.1
 */
@Component
public class VersionComparator
    implements Comparator<String>
{
  private static final GenericVersionScheme VERSION_SCHEME = new GenericVersionScheme();

  public static final Comparator<String> INSTANCE = new VersionComparator();

  private static final Pattern VERSION_RE = Pattern.compile("^\\d+([._-][0-9a-z]+)*$", Pattern.CASE_INSENSITIVE);

  /**
   * Parses out Aether version from a string.
   */
  public static Version version(final String version) {
    try {
      return VERSION_SCHEME.parseVersion(version);
    }
    catch (InvalidVersionSpecificationException e) {
      throw new IllegalArgumentException(e);
    }
  }

  @Override
  public int compare(final String o1, final String o2) {
    boolean firstObjectLooksLikeVersion = isVersionLike(o1);
    boolean secondObjectLooksLikeVersion = isVersionLike(o2);

    if (firstObjectLooksLikeVersion ^ secondObjectLooksLikeVersion) {
      if (firstObjectLooksLikeVersion) {
        return 1;
      }
      else {
        return -1;
      }
    }
    else if (firstObjectLooksLikeVersion) {
      Version v1 = version(o1);
      Version v2 = version(o2);
      return v1.compareTo(v2);
    }
    else {
      return compareNotVersionLikeStrings(o1, o2);
    }
  }

  protected int compareNotVersionLikeStrings(final String o1, final String o2) {
    return o1.compareTo(o2);
  }

  protected boolean isVersionLike(final String version) {
    return VERSION_RE.matcher(version).matches();
  }
}
