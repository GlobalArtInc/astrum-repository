
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
 * Validate that a string is a valid Hostname according to RFC-1123. Blank strings will be counted as valid.
 *
 * @since 3.3
 */
@Target({METHOD, FIELD, ANNOTATION_TYPE, CONSTRUCTOR, PARAMETER})
@Retention(RUNTIME)
@Constraint(validatedBy = HostnameValidator.class)
@Documented
public @interface Hostname
{
  String message() default "{org.globalart.astrum.validation.constraint.hostname}";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
