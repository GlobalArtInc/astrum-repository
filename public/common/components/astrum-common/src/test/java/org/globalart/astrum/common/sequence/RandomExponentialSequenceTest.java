
package org.globalart.astrum.common.sequence;

import org.hamcrest.Matchers;
import org.junit.Assert;
import org.junit.Test;

/**
 * Tests [@link RandomExponentialSequence}
 */
public class RandomExponentialSequenceTest
{
  @Test
  public void noRandomness() {
    RandomExponentialSequence seq = RandomExponentialSequence.builder()
        .start(1)
        .factor(2)
        .maxDeviation(0.0f)
        .build();

    Assert.assertThat(seq.next(), Matchers.is(1L));
    Assert.assertThat(seq.next(), Matchers.is(2L));
    Assert.assertThat(seq.next(), Matchers.is(4L));
    Assert.assertThat(seq.next(), Matchers.is(8L));
    Assert.assertThat(seq.next(), Matchers.is(16L));
  }

  @Test
  public void fullRandomness() {
    for (int i = 0; i < 1000; i++) {
      RandomExponentialSequence seq = RandomExponentialSequence.builder()
          .start(10)
          .factor(2)
          .maxDeviation(2.0f)
          .build();

      Assert.assertThat(seq.next(), Matchers.is(10L));
      long n = seq.next();
      Assert.assertThat(n, Matchers.greaterThanOrEqualTo(10L));
      Assert.assertThat(n, Matchers.lessThanOrEqualTo(40L));
    }
  }
}
