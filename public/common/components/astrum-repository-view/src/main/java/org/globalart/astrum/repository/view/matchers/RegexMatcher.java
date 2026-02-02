
package org.globalart.astrum.repository.view.matchers;

import java.util.regex.MatchResult;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Regular expression string matcher.
 *
 * @since 3.0
 */
public class RegexMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public interface State
  {
    MatchResult getMatchResult();
  }

  private final Pattern pattern;

  public RegexMatcher(final Pattern pattern) {
    this.pattern = checkNotNull(pattern);
  }

  public RegexMatcher(final String regex, final int flags) {
    checkNotNull(regex);
    this.pattern = Pattern.compile(regex, flags);
  }

  public RegexMatcher(final String regex) {
    checkNotNull(regex);
    this.pattern = Pattern.compile(regex);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);

    String path = context.getRequest().getPath();
    log.debug("Matching: {}~={}", path, pattern);
    final java.util.regex.Matcher m = pattern.matcher(path);

    if (m.matches()) {
      // expose match result in context
      context.getAttributes().set(State.class, new State()
      {
        @Override
        public MatchResult getMatchResult() {
          return m;
        }
      });
      return true;
    }

    // no match
    return false;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "pattern=" + pattern +
        '}';
  }
}
