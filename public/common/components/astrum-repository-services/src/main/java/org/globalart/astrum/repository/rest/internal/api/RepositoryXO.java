
package org.globalart.astrum.repository.rest.internal.api;

/**
 * @since 3.29
 */
public class RepositoryXO
{
  private final String id;

  private final String name;

  public RepositoryXO(final String id, final String name) {
    this.id = id;
    this.name = name;
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }
}
