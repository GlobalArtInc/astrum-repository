
package org.globalart.astrum.internal.security.secrets;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.secrets.SecretData;
import org.globalart.astrum.crypto.secrets.SecretsStore;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;
import org.springframework.stereotype.Component;

@Component
public class SecretsStoreImpl
    extends ConfigStoreSupport<SecretsDAO>
    implements SecretsStore
{
  @Autowired
  public SecretsStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, SecretsDAO.class);
  }

  @Transactional
  @Override
  public int create(
      final String purpose,
      @Nullable final String keyId,
      final String secret,
      @Nullable final String userId)
  {
    SecretData secretData = new SecretData();
    secretData.setPurpose(purpose);
    secretData.setKeyId(keyId);
    secretData.setSecret(secret);
    secretData.setUserId(userId);
    dao().create(secretData);
    return secretData.getId();
  }

  @Transactional
  @Override
  public boolean delete(final int id) {
    return dao().delete(id) > 0;
  }

  @Transactional
  @Override
  public boolean update(final int id, final String oldSecret, final String keyId, final String secret) {
    return dao().update(id, oldSecret, keyId, secret) > 0;
  }

  @Transactional
  @Override
  public Optional<SecretData> read(final int id) {
    return dao().read(id);
  }

  @Transactional
  @Override
  public boolean existWithDifferentKeyId(final String keyId) {
    return dao().existWithDifferentKeyId(keyId);
  }

  @Transactional
  @Override
  public List<SecretData> fetchWithDifferentKeyId(final String keyId, final int limit) {
    return dao().fetchWithDifferentKeyId(keyId, limit);
  }
}
