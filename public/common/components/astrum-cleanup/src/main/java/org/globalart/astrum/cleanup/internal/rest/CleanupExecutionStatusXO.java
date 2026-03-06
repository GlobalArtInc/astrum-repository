
package org.globalart.astrum.cleanup.internal.rest;

public class CleanupExecutionStatusXO
{
  public enum Status
  {
    RUNNING,
    COMPLETED,
    FAILED
  }

  private String id;

  private String repository;

  private String policy;

  private Status status;

  private Long componentsDeleted;

  private Long componentCount;

  private Long durationMs;

  private String error;

  private boolean dryRun;

  public CleanupExecutionStatusXO() {
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
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

  public Status getStatus() {
    return status;
  }

  public void setStatus(final Status status) {
    this.status = status;
  }

  public Long getComponentsDeleted() {
    return componentsDeleted;
  }

  public void setComponentsDeleted(final Long componentsDeleted) {
    this.componentsDeleted = componentsDeleted;
  }

  public Long getComponentCount() {
    return componentCount;
  }

  public void setComponentCount(final Long componentCount) {
    this.componentCount = componentCount;
  }

  public Long getDurationMs() {
    return durationMs;
  }

  public void setDurationMs(final Long durationMs) {
    this.durationMs = durationMs;
  }

  public String getError() {
    return error;
  }

  public void setError(final String error) {
    this.error = error;
  }

  public boolean isDryRun() {
    return dryRun;
  }

  public void setDryRun(final boolean dryRun) {
    this.dryRun = dryRun;
  }
}
