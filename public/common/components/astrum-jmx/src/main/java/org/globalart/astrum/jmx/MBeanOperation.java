
package org.globalart.astrum.jmx;

import javax.annotation.Nullable;
import javax.management.MBeanOperationInfo;

/**
 * MBean operation.
 *
 * @since 3.0
 */
public interface MBeanOperation
    extends MBeanFeature
{
  /**
   * Operation information.
   */
  MBeanOperationInfo getInfo();

  /**
   * Operation name.
   */
  String getName();

  /**
   * Operation key.
   */
  OperationKey getKey();

  /**
   * Invoke operation.
   */
  @Nullable
  Object invoke(Object[] params) throws Exception;
}
