
package org.globalart.astrum.security.config;

/**
 * Pre-configured security model configuration source.
 *
 * @since 3.0
 */
public class PreconfiguredSecurityConfigurationSource
    implements SecurityConfigurationSource
{
  private final SecurityConfiguration configuration;

  public PreconfiguredSecurityConfigurationSource(final SecurityConfiguration configuration) {
    this.configuration = configuration;
  }

  @Override
  public SecurityConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public SecurityConfiguration loadConfiguration() {
    return getConfiguration();
  }
}
