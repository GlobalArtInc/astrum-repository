
package org.globalart.astrum.security.internal;

import java.util.Random;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

import org.apache.shiro.crypto.hash.Hash;
import org.apache.shiro.crypto.hash.HashSpi;

/**
 * Test-only {@link HashSpi} registered via {@code META-INF/services} so it is discovered by
 * {@link CachingHashService}'s {@link java.util.ServiceLoader} scan. It implements a synthetic algorithm that no real
 * request uses, and counts how many times it is instantiated — letting a test prove that provider discovery happens
 * once per {@link CachingHashService} instance rather than on every {@code computeHash} call (the regression the class
 * fixes). Its hashing methods are never invoked for the synthetic algorithm.
 */
public class CountingHashSpi
    implements HashSpi
{
  static final String ALGORITHM = "COUNTING-TEST";

  private static final AtomicInteger INSTANTIATIONS = new AtomicInteger();

  public CountingHashSpi() {
    INSTANTIATIONS.incrementAndGet();
  }

  static int instantiations() {
    return INSTANTIATIONS.get();
  }

  @Override
  public Set<String> getImplementedAlgorithms() {
    return Set.of(ALGORITHM);
  }

  @Override
  public Hash fromString(final String format) {
    throw new UnsupportedOperationException("CountingHashSpi is discovery-only");
  }

  @Override
  public HashFactory newHashFactory(final Random random) {
    throw new UnsupportedOperationException("CountingHashSpi is discovery-only");
  }
}
