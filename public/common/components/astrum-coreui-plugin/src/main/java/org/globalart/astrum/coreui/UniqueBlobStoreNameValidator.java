
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Ensure that BlobStore names are unique case-insensitively.
 *
 * @since 3.1
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueBlobStoreNameValidator
    extends ConstraintValidatorSupport<UniqueBlobStoreName, String>
{

  private final BlobStoreManager blobStoreManager;

  @Autowired
  public UniqueBlobStoreNameValidator(final BlobStoreManager blobStoreManager) {
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  @Override
  public boolean isValid(final String s, final ConstraintValidatorContext constraintValidatorContext) {
    return !blobStoreManager.exists(s);
  }
}
