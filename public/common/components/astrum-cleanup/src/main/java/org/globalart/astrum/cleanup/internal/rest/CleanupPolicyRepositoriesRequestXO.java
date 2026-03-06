
package org.globalart.astrum.cleanup.internal.rest;

import java.util.List;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

public class CleanupPolicyRepositoriesRequestXO
{
  @NotNull
  private List<@NotBlank String> repositories;

  public CleanupPolicyRepositoriesRequestXO() {
  }

  public List<String> getRepositories() {
    return repositories;
  }

  public void setRepositories(final List<String> repositories) {
    this.repositories = repositories;
  }
}
