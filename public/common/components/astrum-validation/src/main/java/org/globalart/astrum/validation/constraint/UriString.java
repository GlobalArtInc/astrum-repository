
package org.globalart.astrum.validation.constraint;

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
 * Validate a URI String.
 *
 * @since 3.21
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UriStringValidator.class)
public @interface UriString
{
  String message() default "URI should be in the format 'scheme:[//authority][/path][?query][#fragment]'";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
