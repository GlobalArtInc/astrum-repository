
package org.globalart.astrum.security.authz;

import java.util.Date;

import org.globalart.astrum.security.ClientInfo;

/**
 * Event fired in case of an authorization is tried against given resource.
 */
public class AstrumAuthorizationEvent
{
  private final ClientInfo clientInfo;

  private final ResourceInfo resourceInfo;

  private final boolean successful;

  private final Date date;

  public AstrumAuthorizationEvent(final ClientInfo info, final ResourceInfo resInfo, final boolean successful) {
    this.clientInfo = info;
    this.resourceInfo = resInfo;
    this.successful = successful;
    this.date = new Date();
  }

  public ClientInfo getClientInfo() {
    return clientInfo;
  }

  public ResourceInfo getResourceInfo() {
    return resourceInfo;
  }

  public boolean isSuccessful() {
    return successful;
  }

  public Date getEventDate() {
    return date;
  }
}
