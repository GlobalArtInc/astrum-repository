
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Literal string matcher.
 *
 * @since 3.0
 */
public class LiteralMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String literal;

  private final boolean ignoreCase;

  public LiteralMatcher(final String literal, final boolean ignoreCase) {
    this.literal = checkNotNull(literal);
    this.ignoreCase = ignoreCase;
  }

  public LiteralMatcher(final String literal) {
    this(literal, true);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);
    String path = context.getRequest().getPath();
    log.debug("Matching: {}={} ignore-case: {}", path, literal, ignoreCase);
    if (ignoreCase) {
      return path.equalsIgnoreCase(literal);
    }
    else {
      return path.equals(literal);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "literal='" + literal + '\'' +
        ", ignoreCase=" + ignoreCase +
        '}';
  }
}
