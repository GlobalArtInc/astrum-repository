
package org.globalart.astrum.internal.security.secrets;

import org.globalart.astrum.crypto.secrets.EncryptionKeyValidator;

import com.codahale.metrics.health.HealthCheck;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A {@link HealthCheck} which fails if the administrator has not configured a key to use for encrypting secrets.
 */
@ConditionalOnProperty(name = "astrum.health.check.encryption", havingValue = "true", matchIfMissing = true)
@Component
@Qualifier("Default Secret Encryption Key")
public class DefaultEncryptionKeyHealthCheck
    extends HealthCheck
{
  private static final String TEMPLATE_HEALTHY = "Astrum was configured to use %s to encrypt secrets.";

  private static final String FAIL = "Astrum was not configured with an encryption key and is using the Default key.";

  private final EncryptionKeyValidator encryptionKeyValidator;

  @Autowired
  public DefaultEncryptionKeyHealthCheck(final EncryptionKeyValidator encryptionKeyValidator) {
    this.encryptionKeyValidator = checkNotNull(encryptionKeyValidator);
  }

  @Override
  protected Result check() throws Exception {
    return encryptionKeyValidator.getActiveKeyId()
        .map(DefaultEncryptionKeyHealthCheck::createHealthyMessage)
        .map(Result::healthy)
        .orElseGet(() -> Result.unhealthy(FAIL));
  }

  private static String createHealthyMessage(final String keyId) {
    return String.format(TEMPLATE_HEALTHY, keyId);
  }
}
