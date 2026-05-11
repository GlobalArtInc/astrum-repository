
package org.globalart.astrum.rapture;

import org.globalart.astrum.rapture.settings.RaptureSettings;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Rapture.
 *
 * @since 3.0
 */
@Component
public class UiSettingsManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private RaptureSettings settings = new RaptureSettings();

  public RaptureSettings getSettings() {
    return settings;
  }

  public void setSettings(final RaptureSettings settings) {
    this.settings = checkNotNull(settings);
  }

  public void resetSettings() {
    this.settings = new RaptureSettings();
  }
}
