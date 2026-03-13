
package org.globalart.astrum.repository.search.sql.query.security;

/**
 * PostgresSQL-specific implementation of CselToExpression.
 *
 * PostgresSQL stores paths as brace-delimited strings with quotes:
 * Example: {'/org/apache/commons/commons-compress/1.26.1/commons-compress-1.26.1.pom'}
 * {'/org/apache/commons/commons-compress/1.26.1/commons-compress-1.26.1.jar'}
 */
public class PostgresCselToExpression
    extends CselToExpression
{
  private static final String POSTGRES_TOKEN_START_REGEX = "(^|{')";

  private static final String POSTGRES_TOKEN_END_REGEX = "('}|$)";

  @Override
  protected String getTokenStartRegex() {
    return POSTGRES_TOKEN_START_REGEX;
  }

  @Override
  protected String getTokenEndRegex() {
    return POSTGRES_TOKEN_END_REGEX;
  }
}
