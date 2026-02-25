
package org.globalart.astrum.repository.search.sql.query.syntax;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Represents a {@link StringTerm} which should be treated as having a trailing wildcard
 *
 * @param term the term, must not be null
 * @param isAllowTokenization whether the provided term should be allowed to be tokenized in a full text search
 */
public record WildcardTerm(String term, boolean isAllowTokenization)
    implements StringTerm
{
  public WildcardTerm {
    checkNotNull(term);
  }

  public WildcardTerm(final String term) {
    this(term, true);
  }

  @Override
  public String get() {
    return term;
  }
}
