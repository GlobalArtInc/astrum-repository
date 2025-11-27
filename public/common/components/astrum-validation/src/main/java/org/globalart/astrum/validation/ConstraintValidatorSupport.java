
package org.globalart.astrum.validation;

import java.lang.annotation.Annotation;

import jakarta.validation.ConstraintValidator;

import org.globalart.astrum.common.template.EscapeHelper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Support for {@link ConstraintValidator} implementations.
 *
 * @since 3.0
 */
public abstract class ConstraintValidatorSupport<A extends Annotation, T>
    implements ConstraintValidator<A, T>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final EscapeHelper escapeHelper = new EscapeHelper();

  @Override
  public void initialize(final A constraintAnnotation) {
    // empty
  }

  protected EscapeHelper getEscapeHelper() {
    return escapeHelper;
  }
}
