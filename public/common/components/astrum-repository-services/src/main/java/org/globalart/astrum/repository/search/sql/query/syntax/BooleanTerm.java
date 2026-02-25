
package org.globalart.astrum.repository.search.sql.query.syntax;

import static com.google.common.base.Preconditions.checkNotNull;

public record BooleanTerm(Boolean term)
    implements SingleValueTerm<Boolean>
{
  public BooleanTerm {
    checkNotNull(term);
  }

  @Override
  public Boolean get() {
    return term;
  }
}
