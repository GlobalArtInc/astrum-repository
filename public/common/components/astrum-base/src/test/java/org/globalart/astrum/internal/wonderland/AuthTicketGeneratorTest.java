
package org.globalart.astrum.internal.wonderland;

import org.globalart.astrum.crypto.internal.CryptoHelperImpl;
import org.globalart.astrum.crypto.internal.RandomBytesGeneratorImpl;

import org.junit.Test;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.junit.Assert.assertNotNull;

/**
 * Tests for {@link AuthTicketGenerator}.
 */
public class AuthTicketGeneratorTest
{
  private static final Logger log = LoggerFactory.getLogger(AuthTicketGeneratorTest.class);

  @Test
  public void generateWithDefault() {
    AuthTicketGenerator generator =
        new AuthTicketGenerator(new RandomBytesGeneratorImpl(new CryptoHelperImpl(false)), 16);
    String token = generator.generate();
    log.info(token);
    assertNotNull(token);
  }

  private void generateWithSize(final int size) {
    AuthTicketGenerator generator =
        new AuthTicketGenerator(new RandomBytesGeneratorImpl(new CryptoHelperImpl(false)), 16);
    String token = generator.generate(size);
    log.info(token);
    assertNotNull(token);
  }

  @Test
  public void generateWithSize32() {
    generateWithSize(32);
  }

  @Test
  public void generateWithSize66() {
    generateWithSize(66);
  }
}
