
package org.globalart.astrum.validation;

import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link ConstraintViolation} helpers.
 *
 * @since 3.0
 */
public class ConstraintViolations
{
  private ConstraintViolations() {
    // empty
  }

  private static final Logger log = LoggerFactory.getLogger(ConstraintViolations.class);

  /**
   * Propagate {@link ConstraintViolationException} if there are any violations.
   *
   * @throws ConstraintViolationException
   */
  public static void maybePropagate(final Set<? extends ConstraintViolation<?>> violations, final Logger log) {
    checkNotNull(violations);
    checkNotNull(log);

    if (!violations.isEmpty()) {
      String message = String.format("Validation failed; %d constraints violated", violations.size());

      if (log.isWarnEnabled()) {
        StringBuilder buff = new StringBuilder();
        int c = 0;
        for (ConstraintViolation<?> violation : violations) {
          buff.append("  ")
              .append(++c)
              .append(") ")
              .append(violation.getMessage())
              .append(", type: ")
              .append(violation.getRootBeanClass())
              .append(", property: ")
              .append(violation.getPropertyPath())
              .append(", value: ")
              .append(violation.getInvalidValue())
              .append(System.lineSeparator());
        }
        log.warn("{}:{}{}", message, System.lineSeparator(), buff);
      }

      throw new ConstraintViolationException(message, violations);
    }
  }

  /**
   * If any non-null violations are passed in, will add them to the provided violations set
   *
   * @param violations set to add items to
   * @param toAdd list of potential items to add
   */
  public static void maybeAdd(final Set<ConstraintViolation<?>> violations, final ConstraintViolation<?>... toAdd) {
    for (ConstraintViolation<?> violation : toAdd) {
      if (violation != null) {
        violations.add(violation);
      }
    }
  }

  /**
   * Propagate {@link ConstraintViolationException} if there are any violations.
   *
   * @throws ConstraintViolationException
   */
  public static void maybePropagate(final Set<? extends ConstraintViolation<?>> violations) {
    maybePropagate(violations, log);
  }
}
