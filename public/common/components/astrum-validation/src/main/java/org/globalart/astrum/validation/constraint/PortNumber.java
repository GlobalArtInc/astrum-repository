
package org.globalart.astrum.validation.constraint;

import java.lang.annotation.Documented;
import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.ANNOTATION_TYPE;
import static java.lang.annotation.ElementType.CONSTRUCTOR;
import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Validate number is a valid port-number.
 *
 * @since 3.0
 */
@Target({METHOD, FIELD, ANNOTATION_TYPE, CONSTRUCTOR, PARAMETER})
@Retention(RUNTIME)
@Constraint(validatedBy = PortNumberValidator.class)
@Documented
public @interface PortNumber
{
  String message() default "{org.globalart.astrum.validation.constraint.portnumber}";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};

  int min() default 1;

  int max() default 65535;
}
