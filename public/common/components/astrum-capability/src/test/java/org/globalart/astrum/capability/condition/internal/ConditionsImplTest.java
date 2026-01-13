
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.condition.CapabilityConditions;
import org.globalart.astrum.capability.condition.LogicalConditions;
import org.globalart.astrum.capability.condition.AstrumConditions;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;

/**
 * {@link ConditionsImpl} UTs.
 *
 * @since capabilities 2.0
 */
public class ConditionsImplTest
{

  /**
   * Passed in factories are returned.
   */
  @Test
  public void and01() {
    final LogicalConditions logicalConditions = mock(LogicalConditions.class);
    final CapabilityConditions capabilityConditions = mock(CapabilityConditions.class);
    final AstrumConditions astrumConditions = mock(AstrumConditions.class);
    final ConditionsImpl underTest = new ConditionsImpl(
        logicalConditions, capabilityConditions, astrumConditions);
    assertThat(underTest.logical(), is(equalTo(logicalConditions)));
    assertThat(underTest.capabilities(), is(equalTo(capabilityConditions)));
    assertThat(underTest.astrum(), is(equalTo(astrumConditions)));
  }

}
