
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.ConditionEvent;
import org.globalart.astrum.capability.condition.EventManagerTestSupport;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * {@link InversionCondition} UTs.
 *
 * @since capabilities 2.0
 */
public class InversionConditionTest
    extends EventManagerTestSupport
{

  @Mock
  private Condition condition;

  private InversionCondition underTest;

  @Before
  public final void setUpInversionCondition() throws Exception {
    underTest = new InversionCondition(eventManager, condition);
    underTest.bind();

    verify(eventManager).register(underTest);
  }

  /**
   * Condition is satisfied initially (because mock returns false).
   */
  @Test
  public void not01() {
    assertThat(underTest.isSatisfied(), is(true));
  }

  /**
   * Condition is not satisfied when negated is satisfied.
   */
  @Test
  public void not02() {
    when(condition.isSatisfied()).thenReturn(true);
    underTest.handle(new ConditionEvent.Satisfied(condition));
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
  }

  /**
   * Condition is satisfied when negated is not satisfied.
   */
  @Test
  public void not03() {
    when(condition.isSatisfied()).thenReturn(false);
    underTest.handle(new ConditionEvent.Unsatisfied(condition));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Event bus handler is removed when releasing.
   */
  @Test
  public void releaseRemovesItselfAsHandler() {
    underTest.release();

    verify(eventManager).unregister(underTest);
  }

}
