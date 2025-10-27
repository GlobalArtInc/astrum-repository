
package org.globalart.astrum.common.i18n;

import javax.annotation.Nullable;

/**
 * Provides simple access to i18n messages.
 *
 */
public interface MessageSource
{
  /**
   * @throws ResourceNotFoundException
   */
  String getMessage(String code);

  String getMessage(String code, @Nullable String defaultValue);

  /**
   * @throws ResourceNotFoundException
   */
  String format(String code, Object... args);
}
