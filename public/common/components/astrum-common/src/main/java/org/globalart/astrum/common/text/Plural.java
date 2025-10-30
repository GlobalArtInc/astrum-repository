
package org.globalart.astrum.common.text;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper to render plural strings.
 *
 * @since 3.0
 */
public final class Plural
{
  private Plural() {
    // empty
  }

  public static StringBuilder append(
      final StringBuilder buff,
      final int value,
      final String singular,
      @Nullable final String plural)
  {
    checkNotNull(buff);
    checkNotNull(singular);
    buff.append(value).append(" ");
    if (value == 1) {
      buff.append(singular);
    }
    else {
      if (plural == null) {
        buff.append(singular).append("s");
      }
      else {
        buff.append(plural);
      }
    }
    return buff;
  }

  public static StringBuilder append(
      final StringBuilder buff,
      final int value,
      final String singular)
  {
    return append(buff, value, singular, null);
  }

  public static String of(final int value, final String singular, @Nullable final String plural) {
    return append(new StringBuilder(), value, singular, plural).toString();
  }

  public static String of(final int value, final String singular) {
    return of(value, singular, null);
  }
}
