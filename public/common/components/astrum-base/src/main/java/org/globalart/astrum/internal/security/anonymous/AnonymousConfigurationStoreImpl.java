
package org.globalart.astrum.internal.security.anonymous;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link AnonymousConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Primary
@Component
@Qualifier("mybatis")
public class AnonymousConfigurationStoreImpl
    extends ConfigStoreSupport<AnonymousConfigurationDAO>
    implements AnonymousConfigurationStore
{
  @Autowired
  public AnonymousConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public AnonymousConfiguration newConfiguration() {
    return new AnonymousConfigurationData();
  }

  @Transactional
  @Override
  public AnonymousConfiguration load() {
    return dao().get().orElse(null);
  }

  @Transactional
  @Override
  public void save(final AnonymousConfiguration configuration) {
    postCommitEvent(() -> new AnonymousConfigurationUpdatedEvent((AnonymousConfigurationData) configuration));
    dao().set((AnonymousConfigurationData) configuration);
  }
}
