
package org.globalart.astrum.common.collect;

import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper to build attribute keys.
 *
 * @since 3.0
 */
public class AttributeKey
{
  @VisibleForTesting
  static final String SUFFIX_SEPARATOR = "#";

  private AttributeKey() {
    // empty
  }

  public static String get(final Class<?> type) {
    checkNotNull(type);
    return type.getName();
  }

  public static String get(final Class<?> type, final String suffix) {
    checkNotNull(suffix);
    return get(type) + SUFFIX_SEPARATOR + suffix;
  }
}
