
package org.globalart.astrum.security.role;

import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.ANNOTATION_TYPE;
import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Validate role does not contain itself anywhere in the child hierarchy.
 *
 * @since 3.10
 */
@Target({TYPE, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = RoleNotContainSelfValidator.class)
public @interface RoleNotContainSelf
{
  String message() default "A role cannot contain itself directly or indirectly through other roles.";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};

  String id();

  String roleIds();
}
