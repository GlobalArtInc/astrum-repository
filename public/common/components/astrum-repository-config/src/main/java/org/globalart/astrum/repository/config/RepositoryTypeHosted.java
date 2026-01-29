
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
 * Validate a repository type is hosted
 *
 * @since 3.31
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = RepositoryTypeHostedValidator.class)
public @interface RepositoryTypeHosted
{
  String message() default "must be a hosted type repository name";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
