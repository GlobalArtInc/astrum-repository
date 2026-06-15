
package org.globalart.astrum.coreui;

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
 * Validate a name as unique across all BlobStores.
 *
 * @since 3.1
 */
@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = UniqueBlobStoreNameValidator.class)
public @interface UniqueBlobStoreName
{
  String message() default "Name is already used, must be unique (ignoring case)";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
