
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
 * Validates that a URL follows configuration for accessing private/local network addresses.
 * Uses SSRF protection to prevent Server-Side Request Forgery attacks.
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = SsrfSafeUrlValidator.class)
public @interface SsrfSafeUrl
{
  String message() default "URL points to a blocked private/local network address";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
