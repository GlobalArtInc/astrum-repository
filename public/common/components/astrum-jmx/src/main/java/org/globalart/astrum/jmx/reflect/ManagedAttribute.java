
package org.globalart.astrum.jmx.reflect;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a method as a managed attribute.
 *
 * @see ManagedObject
 * @see ManagedOperation
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.METHOD})
public @interface ManagedAttribute
{
  /**
   * Customized attribute name. If left unset will detect attribute-name from method.
   */
  String name() default "";

  /**
   * Optional attribute description.
   */
  String description() default "";
}
