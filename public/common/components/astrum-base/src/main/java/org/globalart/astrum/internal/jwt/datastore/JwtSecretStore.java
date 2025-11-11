
package org.globalart.astrum.internal.jwt.datastore;

import java.util.Optional;
import java.util.UUID;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.security.jwt.JwtSecretChanged;
import org.globalart.astrum.security.jwt.SecretStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;

/**
 * Implementation of {@link SecretStore} for datastore.
 *
 * @since 3.38
 */
@Component
@Qualifier("mybatis")
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
public class JwtSecretStore
    extends ConfigStoreSupport<JwtSecretDAO>
    implements SecretStore
{
  @Autowired
  public JwtSecretStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  @Override
  public Optional<String> getSecret() {
    return dao().get();
  }

  @Transactional
  @Override
  public void setSecret(final String secret) {
    postCommitEvent(JwtSecretChanged::new);
    dao().set(secret);
  }

  @Transactional
  @Override
  public void generateNewSecret() {
    String secret = UUID.randomUUID().toString();
    postCommitEvent(JwtSecretChanged::new);
    dao().setIfEmpty(secret);
  }
}
