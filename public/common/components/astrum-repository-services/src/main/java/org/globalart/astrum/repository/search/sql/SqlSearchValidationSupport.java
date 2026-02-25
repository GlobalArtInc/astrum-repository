
package org.globalart.astrum.repository.search.sql;

import java.util.Collection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Support class for SQL search validation
 */
public abstract class SqlSearchValidationSupport
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /*
   * For SQL search we prohibit leading wildcards and short prefixes with wildcards for performance reasons.
   * The minimum prefix length is configurable via the astrum.search.wildcard.minPrefixLength property.
   */
  protected Collection<String> getValidTokens(final Collection<String> tokens) {
    return SqlSearchWildcardValidator.getValidTokens(tokens);
  }
}
