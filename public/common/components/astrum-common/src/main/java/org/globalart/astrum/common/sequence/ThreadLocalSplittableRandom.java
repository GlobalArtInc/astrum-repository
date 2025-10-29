
package org.globalart.astrum.common.sequence;

import java.security.SecureRandom;
import java.util.SplittableRandom;

/**
 * {@link SplittableRandom} that automatically splits itself across threads.
 *
 * @since 3.19
 */
public class ThreadLocalSplittableRandom
    extends ThreadLocal<SplittableRandom>
{
  private final SplittableRandom random = new SplittableRandom(new SecureRandom().nextLong());

  @Override
  protected SplittableRandom initialValue() {
    synchronized (random) {
      return random.split();
    }
  }
}
