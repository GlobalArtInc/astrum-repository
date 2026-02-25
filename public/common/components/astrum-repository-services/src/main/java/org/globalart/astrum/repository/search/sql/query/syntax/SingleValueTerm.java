
package org.globalart.astrum.repository.search.sql.query.syntax;

public interface SingleValueTerm<T>
    extends Term
{
  T get();
}
