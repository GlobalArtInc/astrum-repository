
package org.globalart.astrum.httpclient.config;

import java.lang.annotation.Documented;
import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import jakarta.validation.Constraint;
import jakarta.validation.Payload;

import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Validate Java networking glob-like expression.
 *
 * @since 3.0
 */
@Target({METHOD, FIELD, PARAMETER})
@Retention(RUNTIME)
@Constraint(validatedBy = {NonProxyHostsValidator.class, NonProxyHostsValidator.ForArray.class})
@Documented
public @interface NonProxyHosts
{
  String message() default "Invalid Java http.nonProxyHosts expression";

  Class<?>[] groups() default {};

  Class<? extends Payload>[] payload() default {};
}
