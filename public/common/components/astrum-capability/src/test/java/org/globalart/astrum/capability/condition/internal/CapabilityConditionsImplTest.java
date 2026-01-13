
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.event.EventManager;

import org.hamcrest.Matchers;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

/**
 * {@link CapabilityConditionsImpl} UTs.
 *
 * @since capabilities 2.0
 */
public class CapabilityConditionsImplTest
{

  private CapabilityConditionsImpl underTest;

  @Before
  public final void setUpCapabilityConditions() {
    final EventManager eventManager = mock(EventManager.class);
    final CapabilityDescriptorRegistry descriptorRegistry = mock(CapabilityDescriptorRegistry.class);
    final CapabilityRegistry capabilityRegistry = mock(CapabilityRegistry.class);
    underTest = new CapabilityConditionsImpl(eventManager, descriptorRegistry, capabilityRegistry);
  }

  /**
   * capabilityOfTypeExists() factory method returns expected condition.
   */
  @Test
  public void capabilityOfTypeExists() {
    assertThat(
        underTest.capabilityOfTypeExists(capabilityType("test")),
        is(Matchers.<Condition>instanceOf(CapabilityOfTypeExistsCondition.class)));
  }

  /**
   * capabilityOfTypeActive() factory method returns expected condition.
   */
  @Test
  public void capabilityOfTypeActive() {
    assertThat(
        underTest.capabilityOfTypeActive(capabilityType("test")),
        is(Matchers.<Condition>instanceOf(CapabilityOfTypeActiveCondition.class)));
  }

  /**
   * reactivateCapabilityOnUpdate() factory method returns expected condition.
   */
  @Test
  public void reactivateCapabilityOnUpdate() {
    assertThat(
        underTest.passivateCapabilityDuringUpdate(),
        is(Matchers.<Condition>instanceOf(PassivateCapabilityDuringUpdateCondition.class)));
  }

}
