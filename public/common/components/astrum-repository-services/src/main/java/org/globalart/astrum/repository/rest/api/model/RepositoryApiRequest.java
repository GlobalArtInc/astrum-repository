
package org.globalart.astrum.repository.rest.api.model;

/**
 * @since 3.24
 */
public interface RepositoryApiRequest
{
  String getName();

  String getFormat();

  String getType();

  Boolean getOnline();
}
