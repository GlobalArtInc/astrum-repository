
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.condition.EventManagerTestSupport;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * {@link AstrumIsActiveCondition} UTs.
 *
 * @since capabilities 2.0
 */
public class AstrumIsActiveConditionTest
    extends EventManagerTestSupport
{

  private AstrumIsActiveCondition underTest;

  @Before
  public final void setUpAstrumIsActiveCondition() throws Exception {
    underTest = new AstrumIsActiveCondition(eventManager);
  }

  /**
   * Condition is not satisfied initially.
   */
  @Test
  public void notSatisfiedInitially() {
    assertThat(underTest.isSatisfied(), is(false));
  }

  /**
   * Condition is satisfied when Astrum is started.
   */
  @Test
  public void satisfiedWhenAstrumStarted() {
    underTest.start();
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition is satisfied when negated is not satisfied.
   */
  @Test
  public void unsatisfiedWhenAstrumStopped() {
    underTest.start();
    underTest.stop();
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
  }

}
