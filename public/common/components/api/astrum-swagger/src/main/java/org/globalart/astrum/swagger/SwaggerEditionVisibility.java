
package org.globalart.astrum.swagger;

import java.lang.annotation.Retention;
import java.lang.annotation.Target;

import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Allows hiding fields from the Swagger model, to hide from JSON output use in conjunction with
 * {@code @JsonFilter(SwaggerEditionVisibility.NAME)} on the containing object
 */
@Target({FIELD})
@Retention(RUNTIME)
public @interface SwaggerEditionVisibility
{
  public static final String NAME = "EditionVisibility";

  boolean cloud() default true;

  boolean community() default true;

  boolean pro() default true;

  /**
   * Specify a note about why this field is hidden. Not used at runtime
   */
  String note() default "";
}
