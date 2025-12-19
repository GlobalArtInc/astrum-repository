
package org.globalart.astrum.security.role;

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
 * Validate role-id as unique.
 *
 * @since 3.0
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UniqueRoleIdValidator.class)
public @interface UniqueRoleId
{
  String message() default "Role-ID must be unique";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
