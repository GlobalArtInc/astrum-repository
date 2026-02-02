
package org.globalart.astrum.repository.view.matchers.token;

import java.util.regex.Pattern;

/**
 * A token representing an unchanging portion of a path.
 *
 * @since 3.0
 */
public class LiteralToken
    extends Token
{
  public LiteralToken(final String value) {
    super(value);
  }

  @Override
  public String toRegexp() {
    return Pattern.quote(value);
  }

  public String toString() {
    return String.format("lit(%s))", value);
  }
}
