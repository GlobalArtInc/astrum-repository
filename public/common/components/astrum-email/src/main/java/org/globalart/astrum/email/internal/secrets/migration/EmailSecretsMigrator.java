
package org.globalart.astrum.email.internal.secrets.migration;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.security.secrets.SecretsMigrator;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class EmailSecretsMigrator
    implements SecretsMigrator
{
  private final EmailManager emailManager;

  @Autowired
  public EmailSecretsMigrator(final EmailManager emailManager) {
    this.emailManager = checkNotNull(emailManager);
  }

  @Override
  public void migrate() {
    CancelableHelper.checkCancellation();
    EmailConfiguration configuration = emailManager.getConfiguration();
    Secret password = configuration.getPassword();

    // password not exists or is already migrated
    if (password == null || isPersistedSecret(password)) {
      return;
    }

    // email manager encrypts and handles removal in case of failure
    emailManager.setConfiguration(configuration, new String(password.decrypt()));
  }

}
