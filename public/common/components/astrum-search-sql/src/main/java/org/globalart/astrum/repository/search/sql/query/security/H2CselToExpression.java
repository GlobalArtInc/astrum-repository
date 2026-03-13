
package org.globalart.astrum.repository.search.sql.query.security;

/**
 * H2-specific implementation of CselToExpression.
 *
 * H2 stores paths with square brackets and double-quoted comma separated elements.
 * Example: ["/org/slf4j/slf4j-api/1.5.6/slf4j-api-1.5.6.jar","/org/slf4j/slf4j-api/1.5.6/slf4j-api-1.5.6.pom"]
 */
public class H2CselToExpression
    extends CselToExpression
{
  private static final String H2_TOKEN_START_REGEX = "(\\[\"|,\")";

  private static final String H2_TOKEN_END_REGEX = "(\",|\"\\])";

  @Override
  protected String getTokenStartRegex() {
    return H2_TOKEN_START_REGEX;
  }

  @Override
  protected String getTokenEndRegex() {
    return H2_TOKEN_END_REGEX;
  }
}
