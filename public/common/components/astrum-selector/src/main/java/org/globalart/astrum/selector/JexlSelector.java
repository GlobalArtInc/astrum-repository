
package org.globalart.astrum.selector;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import org.apache.commons.jexl3.JexlContext;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.Boolean.TRUE;

/**
 * {@link Selector} implementation that uses JEXL to evaluate expressions describing the selection criteria.
 *
 * @see <a href="http://commons.apache.org/proper/commons-jexl/">Commons Jexl</a>
 *
 * @since 3.0
 */
public class JexlSelector
    implements Selector
{
  public static final String TYPE = "jexl";

  protected final JexlExpression expression;

  public JexlSelector(final JexlExpression expression) {
    this.expression = checkNotNull(expression);
  }

  @Override
  public boolean evaluate(final VariableSource source) {
    return TRUE.equals(expression.evaluate(asJexlContext(source)));
  }

  @Override
  public void toSql(final SelectorSqlBuilder sqlBuilder) {
    throw new UnsupportedOperationException();
  }

  @Override
  public <T> void toSql(final T sqlBuilder, final CselToSql<T> cselToSql) {
    throw new UnsupportedOperationException();
  }

  @Override
  public String toString() {
    return expression.getParsedText();
  }

  /**
   * Wraps the given {@link VariableSource} so it can be used as a lazy {@link JexlContext}.
   */
  private static JexlContext asJexlContext(final VariableSource source) {
    return new JexlContext()
    {
      private final Set<String> names = source.getVariableSet();

      private final Map<String, Optional<?>> values = new HashMap<>(names.size());

      @Override
      public boolean has(final String name) {
        return names.contains(name);
      }

      @Override
      public Object get(final String name) {
        return values.computeIfAbsent(name, source::get).orElse(null);
      }

      @Override
      public void set(final String name, final Object value) {
        throw new UnsupportedOperationException();
      }
    };
  }
}
