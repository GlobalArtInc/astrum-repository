
package org.globalart.astrum.api.rest.selfhosted.security.eula.model;

public class EulaStatus
{
  public static final String EXPECTED_DISCLAIMER =
      "Use of GlobalArt Astrum Repository - Community Edition is governed by the End User License Agreement at https://links.globalart.dev/products/astrum/ce-eula. By returning the value from ‘accepted:false’ to ‘accepted:true’, you acknowledge that you have read and agree to the End User License Agreement at https://links.globalart.dev/products/astrum/ce-eula.";

  private boolean accepted;

  private String disclaimer;

  public boolean isAccepted() {
    return accepted;
  }

  public void setAccepted(boolean accepted) {
    this.accepted = accepted;
  }

  public String getDisclaimer() {
    return disclaimer;
  }

  public void setDisclaimer(String disclaimer) {
    this.disclaimer = disclaimer;
  }

  public boolean hasExpectedDisclaimer() {
    return disclaimer != null && !disclaimer.isEmpty() && EXPECTED_DISCLAIMER.equals(disclaimer);
  }
}
