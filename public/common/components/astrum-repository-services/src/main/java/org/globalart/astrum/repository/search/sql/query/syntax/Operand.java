
package org.globalart.astrum.repository.search.sql.query.syntax;

/**
 * An operand for use in queries.
 */
public enum Operand
{
  /**
   * An operand indicating that the term should be or'd together
   */
  OR(true),

  /**
   * An operand indicating that the term should be and'd together
   */
  AND(true),

  /**
   * An operand indicating that the term should be equal
   */
  EQ(true),

  /**
   * An operand indicating that the term should not be equal
   */
  NOT_EQ(false),

  /**
   * An operand indicating a regular expression match
   */
  REGEX(false),

  /**
   * An operand indicating that one of the terms should match.
   */
  IN(true),

  ANY(true);

  private boolean multiple;

  Operand(final boolean multiple) {
    this.multiple = multiple;
  }

  /**
   * Indicates whether the operand supports multiple terms
   */
  public boolean supportsMultiple() {
    return multiple;
  }
}
