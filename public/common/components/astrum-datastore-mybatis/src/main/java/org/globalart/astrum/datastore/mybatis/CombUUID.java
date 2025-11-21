
package org.globalart.astrum.datastore.mybatis;

import java.util.SplittableRandom;
import java.util.UUID;

import org.globalart.astrum.common.sequence.ThreadLocalSplittableRandom;

/**
 * Generates COMB-style {@link UUID}s for use as database keys: 6-byte timestamp plus 10-byte randomness.
 *
 * @since 3.19
 */
public class CombUUID
{
  private static final ThreadLocalSplittableRandom randomHolder = new ThreadLocalSplittableRandom();

  private CombUUID() {
    // static utility class
  }

  /**
   * @return a new COMB-style UUID
   */
  public static UUID combUUID() {
    SplittableRandom random = randomHolder.get();

    long hiBits = random.nextLong();
    long loBits = random.nextLong();

    hiBits = (System.currentTimeMillis() << 16) | (hiBits & 0x000000000000FFFF);

    return new UUID(hiBits, loBits);
  }
}
