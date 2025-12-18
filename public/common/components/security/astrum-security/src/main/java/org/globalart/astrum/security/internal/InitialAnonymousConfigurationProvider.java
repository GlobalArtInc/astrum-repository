
package org.globalart.astrum.security.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Initial {@link AnonymousConfiguration} provider.
 *
 * Provides the initial configuration of anonymous configuration for fresh server installations.
 *
 * @since 3.0
 */
@Component
@Qualifier("initial")
public class InitialAnonymousConfigurationProvider
    implements FactoryBean<AnonymousConfiguration>
{
  private final boolean enabled;

  @Autowired
  public InitialAnonymousConfigurationProvider(
      @Value("${astrum.security.default.anonymous:false}") final boolean enabled)
  {
    this.enabled = enabled;
  }

  @Override
  public AnonymousConfiguration getObject() {
    return new InitialAnonymousConfiguration();
  }

  private class InitialAnonymousConfiguration
      implements AnonymousConfiguration
  {
    @Override
    public AnonymousConfiguration copy() {
      return this;
    }

    @Override
    public String getRealmName() {
      return AnonymousConfiguration.DEFAULT_REALM_NAME;
    }

    @Override
    public String getUserId() {
      return AnonymousConfiguration.DEFAULT_USER_ID;
    }

    @Override
    public boolean isEnabled() {
      return enabled;
    }

    @Override
    public void setEnabled(final boolean enabled) {
      throw new UnsupportedOperationException();
    }

    @Override
    public void setRealmName(final String realmName) {
      throw new UnsupportedOperationException();
    }

    @Override
    public void setUserId(final String userId) {
      throw new UnsupportedOperationException();
    }
  }

  @Override
  public Class<?> getObjectType() {
    return AnonymousConfiguration.class;
  }
}
