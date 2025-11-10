
package org.globalart.astrum.crypto.internal;

import org.junit.Before;
import org.junit.Test;

/**
 * Tests for {@link RandomBytesGeneratorImpl}.
 */
public class RandomBytesGeneratorImplTest
{
  private RandomBytesGeneratorImpl generator;

  @Before
  public void setUp() throws Exception {
    this.generator = new RandomBytesGeneratorImpl(new CryptoHelperImpl(false));
  }

  @Test(expected = IllegalArgumentException.class)
  public void sizeZero() {
    generator.generate(0);
  }

  @Test(expected = IllegalArgumentException.class)
  public void sizeNegative() {
    generator.generate(-1);
  }
}
