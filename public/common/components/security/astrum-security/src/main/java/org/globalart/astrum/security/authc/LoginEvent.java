
package org.globalart.astrum.security.authc;

public class LoginEvent
    extends SecurityEvent
{
  public LoginEvent(final String principal, final String realm) {
    super(principal, realm);
  }
}
