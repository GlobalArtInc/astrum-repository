
package org.globalart.astrum.security.authc;

public class LogoutEvent
    extends SecurityEvent
{
  public LogoutEvent(final String principal, final String realm) {
    super(principal, realm);
  }
}
