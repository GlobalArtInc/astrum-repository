
package org.globalart.astrum.internal.webhooks;

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
@Constraint(validatedBy = GlobalWebhookTypeValidator.class)
public @interface GlobalWebhookType
{
  String message() default "Incorrect global webhook type";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
