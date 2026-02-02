
package org.globalart.astrum.repository.view.matchers.logic;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Logical NOT matcher.
 *
 * @since 3.0
 *
 * @see LogicMatchers
 */
public class NotMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Matcher matcher;

  @VisibleForTesting
  public NotMatcher(final Matcher matcher) {
    this.matcher = checkNotNull(matcher);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);
    log.debug("Matching: NOT {}", matcher);
    return !matcher.matches(context);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "matcher=" + matcher +
        '}';
  }
}
