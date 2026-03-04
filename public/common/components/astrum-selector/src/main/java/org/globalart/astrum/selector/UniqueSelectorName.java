
package org.globalart.astrum.selector;

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
 * Validate selector-name as unique.
 *
 * @since 3.0
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UniqueSelectorNameValidator.class)
public @interface UniqueSelectorName
{
  String message() default "Selector-name must be unique";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
