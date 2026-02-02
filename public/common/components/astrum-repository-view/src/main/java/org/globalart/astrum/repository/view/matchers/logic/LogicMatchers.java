
package org.globalart.astrum.repository.view.matchers.logic;

import java.util.Arrays;

import org.globalart.astrum.repository.view.Matcher;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Logic matcher factory.
 *
 * @since 3.0
 */
public class LogicMatchers
{
  private LogicMatchers() {
    // empty
  }

  public static Matcher and(final Matcher... matchers) {
    checkNotNull(matchers);
    checkArgument(matchers.length > 1, "AND requires 2 or more matchers");
    return new AndMatcher(Arrays.asList(matchers));
  }

  public static Matcher or(final Matcher... matchers) {
    checkNotNull(matchers);
    checkArgument(matchers.length > 1, "OR requires 2 or more matchers");
    return new OrMatcher(Arrays.asList(matchers));
  }

  public static Matcher not(final Matcher matcher) {
    return new NotMatcher(matcher);
  }
}
