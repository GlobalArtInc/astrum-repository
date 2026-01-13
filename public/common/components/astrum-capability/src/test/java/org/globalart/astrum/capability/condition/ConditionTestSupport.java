
package org.globalart.astrum.capability.condition;

import org.globalart.astrum.common.event.EventManager;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

/**
 * {@link ConditionSupport} UTs.
 *
 * @since capabilities 2.0
 */
public class ConditionTestSupport
    extends EventManagerTestSupport
{

  private TestCondition underTest;

  @Before
  public final void setUpTestCondition() throws Exception {
    underTest = new TestCondition(eventManager);
    underTest.bind();
  }

  /**
   * On creation, condition is not satisfied.
   */
  @Test
  public void notSatisfiedInitially() {
    assertThat(underTest.isSatisfied(), is(false));
  }

  /**
   * When satisfied set to true, condition is satisfied and notification is sent.
   */
  @Test
  public void satisfied() {
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * When satisfied set to true after condition is already satisfied, condition is satisfied and notification is sent
   * only once.
   */
  @Test
  public void satisfiedOnAlreadySatisfied() {
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * When satisfied set to false after condition is already unsatisfied, condition is unsatisfied and notification
   * is sent only once.
   */
  @Test
  public void unsatisfiedOnAlreadyUnsatisfied() {
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));
    underTest.setSatisfied(false);
    assertThat(underTest.isSatisfied(), is(false));
    underTest.setSatisfied(false);
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
  }

  /**
   * When satisfied set to true after condition is unsatisfied, condition is satisfied and notifications are sent.
   */
  @Test
  public void satisfiedOnUnsatisfied() {
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));
    underTest.setSatisfied(false);
    assertThat(underTest.isSatisfied(), is(false));
    underTest.setSatisfied(true);
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest), satisfied(underTest));
  }

  /**
   * Calling setSatisfied() after release does not send events but sets status.
   */
  @Test
  public void setSatisfiedAfterRelease() {
    underTest.release();
    underTest.setSatisfied(false);
    assertThat(underTest.isSatisfied(), is(false));

    verifyNoEventManagerEvents();
  }

  /**
   * Activation context getter returns the passed in value.
   */
  @Test
  public void getActivationContext() {
    assertThat(underTest.getEventManager(), is(equalTo(eventManager)));
  }

  private static class TestCondition
      extends ConditionSupport
  {

    public TestCondition(final EventManager eventManager) {
      super(eventManager);
    }

    @Override
    protected void doBind() {
      // do nothing
    }

    @Override
    protected void doRelease() {
      // do nothing
    }
  }

}
