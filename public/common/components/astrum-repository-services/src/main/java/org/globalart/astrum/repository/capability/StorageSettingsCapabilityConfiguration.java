
package org.globalart.astrum.repository.capability;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;

import jakarta.validation.constraints.NotBlank;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link StorageSettingsCapability} configuration.
 *
 * @since 3.16
 */
public class StorageSettingsCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public static final String LAST_DOWNLOADED_INTERVAL = "lastDownloadedInterval";

  @NotBlank
  private String lastDownloadedInterval;

  public StorageSettingsCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    this.lastDownloadedInterval = properties.get(LAST_DOWNLOADED_INTERVAL);
  }

  public String getLastDownloadedInterval() {
    return lastDownloadedInterval;
  }

  public void setLastDownloadedInterval(final String lastDownloadedInterval) {
    this.lastDownloadedInterval = lastDownloadedInterval;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "lastDownloadedInterval='" + lastDownloadedInterval + '\'' +
        '}';
  }
}
