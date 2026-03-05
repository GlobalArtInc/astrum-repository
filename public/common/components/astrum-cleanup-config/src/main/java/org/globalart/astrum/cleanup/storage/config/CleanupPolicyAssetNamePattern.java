
package org.globalart.astrum.cleanup.storage.config;

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
 * Validate a regular expression
 *
 * @since 3.19
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = CleanupPolicyAssetNamePatternValidator.class)
public @interface CleanupPolicyAssetNamePattern
{
  String message() default "Pattern is invalid";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
