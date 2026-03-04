
package org.globalart.astrum.selector;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintViolationException;

import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.collect.ImmutableSet;
import org.apache.commons.jexl3.JexlException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static org.globalart.astrum.common.text.Strings2.upper;
import static org.globalart.astrum.selector.CselValidator.validateCselExpression;
import static org.globalart.astrum.selector.JexlEngine.expandExceptionDetail;
import org.springframework.stereotype.Component;

/**
 * Factory for validating and creating selectors.
 *
 * @since 3.16
 */
@Component
public class SelectorFactory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final JexlEngine jexlEngine = new JexlEngine();

  private final ConstraintViolationFactory constraintViolationFactory;

  private final CselToSql cselToSql;

  @Autowired
  public SelectorFactory(final ConstraintViolationFactory constraintViolationFactory, final CselToSql cselToSql) {
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
    this.cselToSql = checkNotNull(cselToSql);
  }

  /**
   * Checks the given expression is valid for the chosen selector type.
   *
   * @throws ConstraintViolationException if the expression is invalid
   */
  public void validateSelector(final String type, final String expression) {
    try {
      switch (type) {
        case JexlSelector.TYPE:
          jexlEngine.parseExpression(expression);
          break;
        case CselSelector.TYPE:
          validateCselExpression(jexlEngine.parseExpression(expression));
          break;
        default:
          throw new IllegalArgumentException("Unknown selector type: " + type);
      }
    }
    catch (Exception e) {
      String detail = format("Invalid %s: %s", upper(type),
          e instanceof JexlException ? expandExceptionDetail((JexlException) e) : e.getMessage());

      log.debug(detail, e);

      throw new ConstraintViolationException(e.getMessage(),
          ImmutableSet.of(constraintViolationFactory.createViolation("expression", detail)));
    }
  }

  /**
   * Creates a new {@link Selector} for the given expression and type.
   */
  public Selector createSelector(final String type, final String expression) {
    boolean shouldTrimLeadingSlash = false;
    switch (type) {
      case JexlSelector.TYPE:
        return new JexlSelector(jexlEngine.buildExpression(expression, shouldTrimLeadingSlash));
      case CselSelector.TYPE:
        return new CselSelector(cselToSql, jexlEngine.buildExpression(expression, shouldTrimLeadingSlash));
      default:
        throw new IllegalArgumentException("Unknown selector type: " + type);
    }
  }
}
