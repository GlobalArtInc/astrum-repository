
package org.globalart.astrum.repository.webhooks;

import java.lang.annotation.Retention;
import java.lang.annotation.Target;
import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.ANNOTATION_TYPE;
import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

@Target({FIELD, METHOD, PARAMETER, ANNOTATION_TYPE})
@Retention(RUNTIME)
@Constraint(validatedBy = RepositoryWebhookTypeValidator.class)
public @interface RepositoryWebhookType
{
  String message() default "Incorrect repository webhook type";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
