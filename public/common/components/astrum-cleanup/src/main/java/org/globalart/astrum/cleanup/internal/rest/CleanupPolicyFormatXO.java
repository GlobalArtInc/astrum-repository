
package org.globalart.astrum.cleanup.internal.rest;

import java.util.Set;

/**
 * @since 3.29
 */
public class CleanupPolicyFormatXO
{
  private String id;

  private String name;

  private Set<String> availableCriteria;

  public CleanupPolicyFormatXO(final String id, final String name, final Set<String> availableCriteria) {
    this.id = id;
    this.name = name;
    this.availableCriteria = availableCriteria;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public void setAvailableCriteria(final Set<String> availableCriteria) {
    this.availableCriteria = availableCriteria;
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }

  public Set<String> getAvailableCriteria() {
    return availableCriteria;
  }
}
