
package org.globalart.astrum.capability;

import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.ANNOTATION_TYPE;
import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Validate capability type existence.
 *
 * @since 3.0
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = CapabilityTypeExistsValidator.class)
public @interface CapabilityTypeExists
{
  String message() default "Missing capability type";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
