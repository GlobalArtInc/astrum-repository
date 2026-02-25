
package org.globalart.astrum.repository.search.sql.query.syntax;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A term which is expected to be matched precisely.
 */
public record ExactTerm(String term)
    implements StringTerm
{
  public ExactTerm {
    checkNotNull(term);
  }

  @Override
  public String get() {
    return term;
  }
}
