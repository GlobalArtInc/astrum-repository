
package org.globalart.astrum.repository.search.sql.query.security;

import java.util.Collection;

import static java.lang.String.join;

/**
 * Exception thrown when unknown repository names are specified in a SearchRequest
 */
public class UnknownRepositoriesException
    extends RuntimeException
{
  private static final long serialVersionUID = 9058704277402150465L;

  public UnknownRepositoriesException(final Collection<String> repositories) {
    super("Unknown repositories: [" + join(",", repositories) + "]");
  }
}
