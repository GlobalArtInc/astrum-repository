
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

/**
 * Never matcher.
 *
 * @since 3.0
 */
public class NeverMatcher
    implements Matcher
{
  @Override
  public boolean matches(final Context context) {
    return false;
  }
}
