
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
 * @since 3.13
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UrlStringValidator.class)
public @interface UrlString
{
  String message() default "URL should be in the format 'http://www.example.com'";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
