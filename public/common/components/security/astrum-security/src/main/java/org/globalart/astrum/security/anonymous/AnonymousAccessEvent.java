
package org.globalart.astrum.security.anonymous;

import java.util.Date;

import org.globalart.astrum.security.ClientInfo;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event fired for an anonymous access of the system.
 * 
 * @since 3.1
 */
public class AnonymousAccessEvent
{
  private final ClientInfo clientInfo;

  private final Date eventDate;

  public AnonymousAccessEvent(final ClientInfo clientInfo, final Date eventDate) {
    this.clientInfo = checkNotNull(clientInfo);
    this.eventDate = checkNotNull(eventDate);
  }

  public ClientInfo getClientInfo() {
    return clientInfo;
  }

  public Date getEventDate() {
    return eventDate;
  }
}
