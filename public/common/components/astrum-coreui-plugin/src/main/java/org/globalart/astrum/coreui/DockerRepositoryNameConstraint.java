
package org.globalart.astrum.coreui;

import java.lang.annotation.Documented;
import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Validates that Docker repository names are lowercase to support path-based routing.
 * Docker specification requires repository names to be lowercase.
 *
 * <p>
 * <strong>Security Rationale:</strong> This constraint is scoped to the {@code Create} validation group.
 * The lowercase requirement is enforced on creation to prevent ambiguous routing paths
 * that could be exploited for path traversal or URL confusion attacks. Existing mixed-case repositories
 * (created before this rule) are exempt on update to maintain backward compatibility; these cannot
 * enable path-based routing without renaming (enforced separately in DockerConnectorFacetImpl).
 */
@Target({TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = DockerRepositoryNameValidator.class)
@Documented
public @interface DockerRepositoryNameConstraint
{
  String message() default "Docker repository names must be lowercase to support path-based routing";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
