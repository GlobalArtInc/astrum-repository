
package org.globalart.astrum.rapture;

/**
 *
 * @since 3.31
 */
public class CapabilityEnabledXO
{
  private boolean enabled;

  public CapabilityEnabledXO(Boolean enabled) {
    this.enabled = enabled;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(boolean enabled) {
    this.enabled = enabled;
  }

  @Override
  public String toString() {
    return "CapabilityEnabledXO{" +
        "enabled=" + enabled +
        '}';
  }
}
