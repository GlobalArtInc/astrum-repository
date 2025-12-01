
package org.globalart.astrum.jmx;

import javax.annotation.Nullable;
import javax.management.MBeanAttributeInfo;

/**
 * MBean attribute.
 *
 * @since 3.0
 */
public interface MBeanAttribute
    extends MBeanFeature
{
  /**
   * Attribute information.
   */
  MBeanAttributeInfo getInfo();

  /**
   * Attribute name.
   */
  String getName();

  /**
   * Get the value of the attribute.
   */
  @Nullable
  Object getValue() throws Exception;

  /**
   * Set the value of the attribute.
   */
  void setValue(@Nullable Object value) throws Exception;
}
