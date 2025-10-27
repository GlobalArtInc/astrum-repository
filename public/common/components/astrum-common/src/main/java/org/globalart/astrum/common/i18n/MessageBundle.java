
package org.globalart.astrum.common.i18n;

import java.lang.annotation.Documented;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marker interface for proxy-based I18n message bundles.
 *
 */
public interface MessageBundle
{
  /**
   * Message key.
   *
   */
  @Retention(RetentionPolicy.RUNTIME)
  @Target(ElementType.METHOD)
  @Documented
  public @interface Key
  {
    String value();
  }

  /**
   * Default message.
   *
   */
  @Retention(RetentionPolicy.RUNTIME)
  @Target(ElementType.METHOD)
  @Documented
  public @interface DefaultMessage
  {
    String value();
  }
}
