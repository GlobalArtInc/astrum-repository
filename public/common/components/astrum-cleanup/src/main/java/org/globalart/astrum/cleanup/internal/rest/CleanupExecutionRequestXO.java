
package org.globalart.astrum.cleanup.internal.rest;

import jakarta.validation.constraints.NotBlank;

public class CleanupExecutionRequestXO
{
  @NotBlank
  private String repository;

  private String policy;

  private boolean dryRun;

  public CleanupExecutionRequestXO() {
  }

  public String getRepository() {
    return repository;
  }

  public void setRepository(final String repository) {
    this.repository = repository;
  }

  public String getPolicy() {
    return policy;
  }

  public void setPolicy(final String policy) {
    this.policy = policy;
  }

  public boolean isDryRun() {
    return dryRun;
  }

  public void setDryRun(final boolean dryRun) {
    this.dryRun = dryRun;
  }
}
