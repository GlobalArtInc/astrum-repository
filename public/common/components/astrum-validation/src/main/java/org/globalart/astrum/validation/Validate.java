
package org.globalart.astrum.validation;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Identifies methods whose arguments and return value require validation.
 * 
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.METHOD})
public @interface Validate
{
  Class<?>[] groups() default {};
}
