
package org.globalart.astrum.repository.view.matchers.logic;

import java.util.List;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Joiner;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Logical AND matcher.
 *
 * @since 3.0
 *
 * @see LogicMatchers
 */
public class AndMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<Matcher> matchers;

  @VisibleForTesting
  public AndMatcher(final List<Matcher> matchers) {
    this.matchers = checkNotNull(matchers);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);
    if (log.isDebugEnabled()) {
      log.debug("Matching: {}", Joiner.on(" AND ").join(matchers));
    }
    for (Matcher matcher : matchers) {
      boolean matched = matcher.matches(context);
      if (!matched) {
        return false;
      }
    }
    return true;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "matchers=" + matchers +
        '}';
  }
}
