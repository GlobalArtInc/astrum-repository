
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;

import org.hamcrest.Matchers;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;

/**
 * {@link AstrumConditionsImpl} UTs.
 *
 * @since capabilities 2.0
 */
public class AstrumConditionsImplTest
{

  /**
   * active() factory method returns expected condition.
   */
  @Test
  public void active() {
    final AstrumIsActiveCondition astrumIsActiveCondition = mock(AstrumIsActiveCondition.class);
    final AstrumConditionsImpl underTest = new AstrumConditionsImpl(astrumIsActiveCondition);

    assertThat(
        underTest.active(),
        is(Matchers.<Condition>instanceOf(AstrumIsActiveCondition.class)));
  }

}
