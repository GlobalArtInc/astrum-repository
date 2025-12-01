
package org.globalart.astrum.jmx.reflect;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import javax.management.MBeanOperationInfo;

/**
 * Marks a method as a managed operation.
 *
 * @see ManagedObject
 * @see ManagedAttribute
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.METHOD})
public @interface ManagedOperation
{
  /**
   * Customized attribute name. If left unset will use name from method.
   */
  String name() default "";

  /**
   * Optional operation impact.
   *
   * @see MBeanOperationInfo#ACTION
   * @see MBeanOperationInfo#ACTION_INFO
   * @see MBeanOperationInfo#INFO
   * @see MBeanOperationInfo#UNKNOWN
   */
  int impact() default MBeanOperationInfo.UNKNOWN;

  /**
   * Optional operation description.
   */
  String description() default "";
}
