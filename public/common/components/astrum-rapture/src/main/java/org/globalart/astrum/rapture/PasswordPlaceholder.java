
package org.globalart.astrum.rapture;

import javax.annotation.Nullable;

import org.globalart.astrum.crypto.secrets.Secret;

import com.google.common.annotations.VisibleForTesting;

/**
 * Helper for password-placeholder data.
 *
 * @since 3.0
 */
public class PasswordPlaceholder
{
  private PasswordPlaceholder() {
    // empty
  }

  /**
   * Token used for passwords that are defined, but which are not transmitted.
   */
  @VisibleForTesting
  static final String VALUE = "#~NXRM~PLACEHOLDER~PASSWORD~#";

  /**
   * Returns password placeholder.
   */
  public static String get() {
    return VALUE;
  }

  /**
   * Returns fake password placeholder unless value is {@code null}.
   */
  @Nullable
  public static String get(@Nullable final String value) {
    if (value != null) {
      return VALUE;
    }
    return null;
  }

  /**
   * Returns fake password placeholder unless value is {@code null}.
   */
  @Nullable
  public static String get(@Nullable final Secret value) {
    if (value != null) {
      return VALUE;
    }
    return null;
  }

  /**
   * Determine if given value is a password placeholder.
   */
  public static boolean is(@Nullable final String value) {
    return VALUE.equals(value);
  }

  /**
   * @param value
   * @return true if the value is not the password placeholder
   */
  public static boolean isNot(@Nullable final String value) {
    return !PasswordPlaceholder.is(value);
  }
}
