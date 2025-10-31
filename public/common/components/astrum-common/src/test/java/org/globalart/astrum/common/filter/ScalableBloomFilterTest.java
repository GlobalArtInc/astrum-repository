
package org.globalart.astrum.common.filter;

import java.util.ArrayList;
import java.util.List;

import org.junit.Test;

import static com.google.common.hash.Funnels.stringFunnel;
import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.UUID.randomUUID;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.lessThan;
import static org.hamcrest.core.Is.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class ScalableBloomFilterTest

{
  private static final double FALSE_POSITIVE_PROBABILITY = 10e-19;

  @Test
  public void shouldIdentifyDuplicates() {
    List<String> added = new ArrayList<>();

    ScalableBloomFilter<String> uniqueFilter = buildFilter();
    for (int i = 1; i <= 10; i++) {
      String value = randomUUID().toString();
      added.add(value);
      assertFalse(uniqueFilter.mightContain(value));
      assertTrue(uniqueFilter.put(value));
    }

    for (String value : added) {
      assertTrue(uniqueFilter.mightContain(value));
      assertFalse(uniqueFilter.put(value));
    }
  }

  @Test
  public void shouldNotReturnFalsePositiveInFirstMillion() {
    ScalableBloomFilter<String> uniqueFilter = buildFilter();

    for (int i = 1; i <= 1000000; i++) {
      assertFalse(uniqueFilter.mightContain(randomUUID().toString()));
      assertTrue(uniqueFilter.put(randomUUID().toString()));
    }

    // 1,000,000 records for this configuration leads to a probability of ~1.6047675107709276E-19 for a false positive.
    assertThat(uniqueFilter.expectedFpp(), is(lessThan(10e-18)));
  }

  private ScalableBloomFilter<String> buildFilter() {
    return new ScalableBloomFilter<>(stringFunnel(UTF_8), 1000000, FALSE_POSITIVE_PROBABILITY);
  }
}
