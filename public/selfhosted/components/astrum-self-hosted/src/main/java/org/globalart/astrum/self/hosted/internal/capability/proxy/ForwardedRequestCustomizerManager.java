
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import java.util.Optional;

import org.globalart.astrum.bootstrap.jetty.CustomForwardedRequestCustomizer;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

/**
 * Manager for the {@link CustomForwardedRequestCustomizer}.
 *
 * <p>
 * This component provides runtime control over forwarded header processing.
 * It's used by the {@link ForwardedRequestCapability} to enable/disable header processing.
 * </p>
 *
 * <p>
 * The manager locates the CustomForwardedRequestCustomizer from the Jetty Server's
 * beans after the application starts.
 * </p>
 */
@Component
public class ForwardedRequestCustomizerManager
{
  private static final Logger log = LoggerFactory.getLogger(ForwardedRequestCustomizerManager.class);

  /**
   * Sets whether forwarded header processing is enabled.
   *
   * @param enabled true to enable processing, false to disable
   */
  public void setEnabled(final boolean enabled) {
    Optional<CustomForwardedRequestCustomizer> instance = CustomForwardedRequestCustomizer.instance();
    if (instance.isEmpty()) {
      if (enabled) {
        throw new IllegalStateException("Unable to detect configured CustomForwardedRequestCustomizer");
      }
      log.info("Forwarded header processing disabled");
      return;
    }

    boolean wasEnabled = instance.get().isEnabled();
    if (wasEnabled != enabled) {
      instance.get().setEnabled(enabled);
      log.info("Forwarded header processing {}", enabled ? "enabled" : "disabled");
    }
  }

  @VisibleForTesting
  boolean isEnabled() {
    return CustomForwardedRequestCustomizer.instance()
        .map(CustomForwardedRequestCustomizer::isEnabled)
        .orElse(false);
  }
}
