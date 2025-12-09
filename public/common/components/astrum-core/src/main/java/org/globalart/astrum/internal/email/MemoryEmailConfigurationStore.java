
package org.globalart.astrum.internal.email;

import javax.annotation.Nullable;
import org.globalart.astrum.email.EmailConfiguration;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * In-memory {@link EmailConfigurationStore}.
 *
 * @since 3.0
 */
@Component
@Qualifier("memory")
@Order(Ordered.LOWEST_PRECEDENCE)
@VisibleForTesting
public class MemoryEmailConfigurationStore
    implements EmailConfigurationStore
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private EmailConfiguration model;

  @Nullable
  @Override
  public synchronized EmailConfiguration load() {
    return model;
  }

  @Override
  public synchronized void save(final EmailConfiguration configuration) {
    this.model = checkNotNull(configuration);
  }

  @Override
  public EmailConfiguration newConfiguration() {
    return new MemoryEmailConfiguration();
  }
}
