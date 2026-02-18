
package org.globalart.astrum.repository.manager.internal;

import java.util.Map;

import javax.annotation.Nullable;
import jakarta.validation.ConstraintViolation;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.ConfigurationValidator;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

@Component
public class BlobStoreExistenceValidator
    implements ConfigurationValidator
{
  private final BlobStoreManager blobStoreManager;

  private final ConstraintViolationFactory constraintViolationFactory;

  @Autowired
  public BlobStoreExistenceValidator(
      final BlobStoreManager blobStoreManager,
      final ConstraintViolationFactory constraintViolationFactory)
  {
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
  }

  @Nullable
  @Override
  public ConstraintViolation<?> validate(final Configuration configuration) {
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    if (attributes != null && attributes.containsKey(STORAGE)) {
      Map<String, Object> storageAttributes = attributes.get(STORAGE);
      if (storageAttributes != null && storageAttributes.containsKey(BLOB_STORE_NAME)) {
        String blobStoreName = (String) storageAttributes.get(BLOB_STORE_NAME);
        if (blobStoreName != null && blobStoreManager.get(blobStoreName) == null) {
          return constraintViolationFactory.createViolation(
              STORAGE + "." + BLOB_STORE_NAME,
              "No blob store exists with the specified name");
        }
      }
    }
    return null;
  }
}
