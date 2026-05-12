
package org.globalart.astrum.rapture.internal.state;

import java.util.ArrayList;
import java.util.List;

/**
 * License exchange object.
 *
 * @since 3.0
 */
public class LicenseXO
{
  private boolean required;

  private boolean installed;

  private boolean valid;

  private int daysToExpiry;

  private List<String> features = new ArrayList<>();

  public boolean isRequired() {
    return required;
  }

  public void setRequired(boolean required) {
    this.required = required;
  }

  public boolean isInstalled() {
    return installed;
  }

  public void setInstalled(boolean installed) {
    this.installed = installed;
  }

  public boolean isValid() {
    return valid;
  }

  public void setValid(boolean valid) {
    this.valid = valid;
  }

  public int getDaysToExpiry() {
    return daysToExpiry;
  }

  public void setDaysToExpiry(int daysToExpiry) {
    this.daysToExpiry = daysToExpiry;
  }

  public List<String> getFeatures() {
    return features;
  }

  public void setFeatures(List<String> features) {
    this.features = features;
  }

  @Override
  public String toString() {
    return "LicenseXO{" +
        "required=" + required +
        ", installed=" + installed +
        ", valid=" + valid +
        ", daysToExpiry=" + daysToExpiry +
        ", features=" + features +
        '}';
  }
}
