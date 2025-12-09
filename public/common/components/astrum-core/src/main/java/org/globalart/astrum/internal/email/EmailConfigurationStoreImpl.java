
package org.globalart.astrum.internal.email;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link EmailConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Primary
@Component
@Qualifier("mybatis")
public class EmailConfigurationStoreImpl
    extends ConfigStoreSupport<EmailConfigurationDAO>
    implements EmailConfigurationStore
{
  @Autowired
  public EmailConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public EmailConfiguration newConfiguration() {
    return new EmailConfigurationData();
  }

  @Transactional
  @Override
  public EmailConfiguration load() {
    return dao().get().orElse(null);
  }

  @Transactional
  @Override
  public void save(final EmailConfiguration configuration) {
    postCommitEvent(EmailConfigurationChanged::new);
    dao().set((EmailConfigurationData) configuration);
  }
}
