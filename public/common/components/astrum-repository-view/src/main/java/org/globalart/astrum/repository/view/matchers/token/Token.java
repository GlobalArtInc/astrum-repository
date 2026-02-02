
package org.globalart.astrum.repository.view.matchers.token;

import java.util.Arrays;
import java.util.List;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A portion of a parsed pattern template.
 *
 * @since 3.0
 */
public abstract class Token
{
  protected final String value;

  protected Token(final String value) {
    this.value = checkNotNull(value);
  }

  public abstract String toRegexp();

  private static final List<Character> REGEXP_CHARS = Arrays.asList('[', ']', '{', '}', '(', ')', '\\', '.');

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof Token)) {
      return false;
    }

    Token token = (Token) o;
    return value.equals(token.value);
  }

  @Override
  public int hashCode() {
    return value.hashCode();
  }
}
