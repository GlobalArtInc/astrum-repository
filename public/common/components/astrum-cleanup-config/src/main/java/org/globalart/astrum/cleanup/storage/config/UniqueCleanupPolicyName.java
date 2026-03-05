
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
 * Validate a name as unique across all Cleanup Policies.
 *
 * @since 3.14
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UniqueCleanupPolicyNameValidator.class)
public @interface UniqueCleanupPolicyName
{
  String message() default "Name is already used, must be unique (ignoring case)";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
