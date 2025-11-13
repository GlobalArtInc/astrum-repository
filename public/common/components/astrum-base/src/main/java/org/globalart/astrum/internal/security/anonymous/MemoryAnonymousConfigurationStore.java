
package org.globalart.astrum.internal.security.anonymous;

import javax.annotation.Nullable;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * In-memory {@link AnonymousConfigurationStore}.
 */
@Component
@Qualifier("memory")
@Order(Ordered.LOWEST_PRECEDENCE)
@VisibleForTesting
public class MemoryAnonymousConfigurationStore
    implements AnonymousConfigurationStore
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private AnonymousConfiguration model;

  @Override
  @Nullable
  public synchronized AnonymousConfiguration load() {
    return model;
  }

  @Override
  public synchronized void save(final AnonymousConfiguration configuration) {
    this.model = checkNotNull(configuration);
  }

  @Override
  public AnonymousConfiguration newConfiguration() {
    return new MemoryAnonymousConfiguration();
  }

  private static class MemoryAnonymousConfiguration
      implements AnonymousConfiguration
  {
    private String realmName;

    private String userId;

    private boolean enabled;

    private MemoryAnonymousConfiguration() {
      // no arg
    }

    @Override
    public AnonymousConfiguration copy() {
      MemoryAnonymousConfiguration configuration = new MemoryAnonymousConfiguration();
      configuration.setEnabled(enabled);
      configuration.setRealmName(realmName);
      configuration.setUserId(userId);
      return configuration;
    }

    @Override
    public String getRealmName() {
      return realmName;
    }

    @Override
    public String getUserId() {
      return userId;
    }

    @Override
    public boolean isEnabled() {
      return enabled;
    }

    @Override
    public void setEnabled(final boolean enabled) {
      this.enabled = enabled;
    }

    @Override
    public void setRealmName(final String realmName) {
      this.realmName = realmName;
    }

    @Override
    public void setUserId(final String userId) {
      this.userId = userId;
    }
  }
}
