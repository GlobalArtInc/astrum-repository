
package org.globalart.astrum.capability.condition.internal;

import java.util.Arrays;
import java.util.Collections;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.condition.EventManagerTestSupport;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

/**
 * {@link CapabilityOfTypeExistsCondition} UTs.
 *
 * @since capabilities 2.0
 */
public class CapabilityOfTypeExistsConditionTest
    extends EventManagerTestSupport
{

  @Mock
  private CapabilityReference ref1;

  @Mock
  private CapabilityReference ref2;

  @Mock
  private CapabilityRegistry capabilityRegistry;

  private CapabilityOfTypeExistsCondition underTest;

  @Before
  public final void setUpCapabilityOfTypeExistsCondition() throws Exception {
    final CapabilityType capabilityType = capabilityType(this.getClass().getName());

    when(ref1.context()).thenReturn(mock(CapabilityContext.class));
    when(ref1.context().type()).thenReturn(capabilityType);

    when(ref2.context()).thenReturn(mock(CapabilityContext.class));
    when(ref2.context().type()).thenReturn(capabilityType);

    final CapabilityDescriptorRegistry descriptorRegistry = mock(CapabilityDescriptorRegistry.class);
    final CapabilityDescriptor descriptor = mock(CapabilityDescriptor.class);

    when(descriptor.name()).thenReturn(this.getClass().getSimpleName());
    when(descriptorRegistry.get(capabilityType)).thenReturn(descriptor);

    underTest = new CapabilityOfTypeExistsCondition(
        eventManager, descriptorRegistry, capabilityRegistry, capabilityType);
    underTest.bind();

    verify(eventManager).register(underTest);
  }

  /**
   * Initially, condition should be unsatisfied.
   */
  @Test
  public void initiallyNotSatisfied() {
    assertThat(underTest.isSatisfied(), is(false));
  }

  /**
   * Condition should become satisfied if an active capability of specified type is added.
   */
  @Test
  public void capabilityOfTypeExists01() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should become satisfied if a non active capability of specified type is added.
   */
  @Test
  public void capabilityOfTypeExists02() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(false);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should not be re-satisfied if a new active capability of specified type is added.
   */
  @Test
  public void capabilityOfTypeExists03() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.Created(ref2));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should remain satisfied if another capability of the specified type is removed.
   */
  @Test
  public void capabilityOfTypeExists04() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.Created(ref2));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref2)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.AfterRemove(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should become unsatisfied when all capabilities have been removed.
   */
  @Test
  public void capabilityOfTypeExists05() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Collections.emptyList()).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.AfterRemove(ref1));
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
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
