
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Suffix string matcher.
 *
 * @since 3.0
 */
public class SuffixMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String suffix;

  private final boolean ignoreCase;

  @VisibleForTesting
  public SuffixMatcher(final String suffix, final boolean ignoreCase) {
    this.suffix = checkNotNull(suffix);
    this.ignoreCase = ignoreCase;
  }

  public SuffixMatcher(final String suffix) {
    this(suffix, false);
  }

  public SuffixMatcher ignoreCase(final boolean ignoreCase) {
    return new SuffixMatcher(suffix, ignoreCase);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);
    String path = context.getRequest().getPath();
    log.debug("Matching: {} ends-with={} ignore-case: {}", path, suffix, ignoreCase);
    if (ignoreCase) {
      return Strings2.lower(path).endsWith(Strings2.lower(suffix));
    }
    else {
      return path.endsWith(suffix);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "suffix='" + suffix + '\'' +
        ", ignoreCase=" + ignoreCase +
        '}';
  }
}
