
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.NotEmpty;

/**
 * Repository status exchange object.
 *
 * @since 3.0
 */
public class RepositoryStatusXO
{
  /**
   * Name of associated Repository.
   */
  @NotEmpty
  private String repositoryName;

  /**
   * Whether or not the repository is online.
   */
  private boolean online;

  /**
   * A description of the status.
   */
  private String description;

  /**
   * A reason for the status.
   */
  private String reason;

  public String getRepositoryName() {
    return repositoryName;
  }

  public void setRepositoryName(String repositoryName) {
    this.repositoryName = repositoryName;
  }

  public boolean isOnline() {
    return online;
  }

  public void setOnline(boolean online) {
    this.online = online;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public String getReason() {
    return reason;
  }

  public void setReason(String reason) {
    this.reason = reason;
  }

  @Override
  public String toString() {
    return "RepositoryStatusXO{" +
        "repositoryName='" + repositoryName + '\'' +
        ", online=" + online +
        ", description='" + description + '\'' +
        ", reason='" + reason + '\'' +
        '}';
  }
}
