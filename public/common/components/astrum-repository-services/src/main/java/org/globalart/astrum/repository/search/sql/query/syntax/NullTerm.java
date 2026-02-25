
package org.globalart.astrum.repository.search.sql.query.syntax;

/**
 * A term with a {@code null} value
 */
public record NullTerm()
    implements StringTerm
{
  public static final NullTerm INSTANCE = new NullTerm();

  @Override
  public String get() {
    return null;
  }
}
