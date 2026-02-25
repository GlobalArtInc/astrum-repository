
package org.globalart.astrum.repository.search.sql.query.syntax;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A term which allows tokenization while matching. This allows {@code foo.bar} to potentially match {@code foo-bar}
 * depending on the underlying search implementation. Unlike {@link WildcardTerm} this is not a prefix search so
 * {@code term=foo} will not match {@code foobar}
 */
public record LenientTerm(String term)
    implements StringTerm
{
  public LenientTerm {
    checkNotNull(term);
  }

  @Override
  public String get() {
    return term;
  }
}
