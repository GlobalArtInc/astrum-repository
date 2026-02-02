
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Prefix string matcher.
 *
 * @since 3.0
 */
public class PrefixMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String prefix;

  private final boolean ignoreCase;

  @VisibleForTesting
  public PrefixMatcher(final String prefix, final boolean ignoreCase) {
    this.prefix = checkNotNull(prefix);
    this.ignoreCase = ignoreCase;
  }

  public PrefixMatcher(final String prefix) {
    this(prefix, false);
  }

  public PrefixMatcher ignoreCase(final boolean ignoreCase) {
    return new PrefixMatcher(prefix, ignoreCase);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);
    String path = context.getRequest().getPath();
    log.debug("Matching: {} starts-with={} ignore-case: {}", path, prefix, ignoreCase);
    if (ignoreCase) {
      return Strings2.lower(path).startsWith(Strings2.lower(prefix));
    }
    else {
      return path.startsWith(prefix);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "prefix='" + prefix + '\'' +
        ", ignoreCase=" + ignoreCase +
        '}';
  }
}
