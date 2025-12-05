
package org.globalart.astrum.internal.app;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;
import org.globalart.astrum.validation.constraint.UrlString;

import jakarta.validation.constraints.NotBlank;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link BaseUrlCapability} configuration.
 *
 * @since 3.0
 */
public class BaseUrlCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public static final String URL = "url";

  @NotBlank
  @UrlString
  private String url;

  public BaseUrlCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    this.url = properties.get(URL);
  }

  public String getUrl() {
    return url;
  }

  public void setUrl(final String url) {
    this.url = url;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "url='" + url + '\'' +
        '}';
  }
}
