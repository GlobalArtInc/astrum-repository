
package org.globalart.astrum.repository.config;

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
 * Validate a repository name as matching an existing repository
 *
 * @since 3.31
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = RepositoryNameValidator.class)
public @interface RepositoryName
{
  String message() default "must match an existing repository";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
