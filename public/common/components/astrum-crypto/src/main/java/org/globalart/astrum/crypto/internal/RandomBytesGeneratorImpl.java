
package org.globalart.astrum.crypto.internal;

import java.security.SecureRandom;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.crypto.RandomBytesGenerator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Default {@link RandomBytesGenerator} implementation.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RandomBytesGeneratorImpl
    implements RandomBytesGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SecureRandom random;

  @Autowired
  public RandomBytesGeneratorImpl(final CryptoHelper crypto) {
    this.random = checkNotNull(crypto).createSecureRandom();
  }

  public byte[] generate(final int size) {
    checkArgument(size > 0);
    byte[] bytes = new byte[size];
    random.nextBytes(bytes);
    return bytes;
  }
}
