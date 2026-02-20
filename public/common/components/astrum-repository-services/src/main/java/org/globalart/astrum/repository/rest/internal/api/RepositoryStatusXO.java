
package org.globalart.astrum.repository.rest.internal.api;

import javax.annotation.Nullable;

/**
 * @since 3.30
 */
public class RepositoryStatusXO
{

  private final boolean online;

  private final String description;

  private final String reason;

  public RepositoryStatusXO(
      final boolean online,
      @Nullable final String description,
      @Nullable final String reason)
  {
    this.online = online;
    this.description = description;
    this.reason = reason;
  }

  public boolean isOnline() {
    return online;
  }

  @Nullable
  public String getDescription() {
    return description;
  }

  @Nullable
  public String getReason() {
    return reason;
  }
}
