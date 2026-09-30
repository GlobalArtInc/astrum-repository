
package org.globalart.astrum.bootstrap.core;

import java.util.Map;

import org.globalart.astrum.common.app.ApplicationLicense;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Component;

import static java.util.Collections.emptyMap;

/**
 * CORE {@link ApplicationLicense}.
 */
@Component
@ConditionalOnProperty(value = "astrum.edition", havingValue = "CORE", matchIfMissing = true)
public class ApplicationLicenseImpl
    implements ApplicationLicense
{
  /**
   * Always {@code false}.
   */
  @Override
  public boolean isRequired() {
    return false;
  }

  /**
   * Always {@code false}.
   */
  @Override
  public boolean isValid() {
    return false;
  }

  /**
   * Always {@code false}.
   */
  @Override
  public boolean isInstalled() {
    return false;
  }

  /**
   * Always {@code false}.
   */
  @Override
  public boolean isExpired() {
    return false;
  }

  /**
   * Always empty-map.
   */
  @Override
  public Map<String, Object> getAttributes() {
    return emptyMap();
  }

  /**
   * Always {@code null}.
   */
  @Override
  @Nullable
  public String getFingerprint() {
    return null;
  }

  @Override
  public void refresh() {
    // no-op
  }

  @Override
  public boolean isEvaluation() {
    return false;
  }
}
