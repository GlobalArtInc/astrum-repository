
package org.globalart.astrum.capability.condition.internal;

import java.util.Arrays;
import java.util.Collections;

import org.globalart.astrum.capability.Capability;
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
 * {@link CapabilityOfTypeActiveCondition} UTs.
 *
 * @since capabilities 2.0
 */
public class CapabilityOfTypeActiveConditionTest
    extends EventManagerTestSupport
{

  @Mock
  private CapabilityReference ref1;

  @Mock
  private CapabilityReference ref2;

  @Mock
  private CapabilityReference ref3;

  @Mock
  private CapabilityRegistry capabilityRegistry;

  private CapabilityOfTypeActiveCondition underTest;

  @Before
  public final void setUpCapabilityOfTypeActiveCondition() throws Exception {
    final CapabilityType capabilityType = capabilityType(this.getClass().getName());

    when(ref1.context()).thenReturn(mock(CapabilityContext.class));
    when(ref1.context().type()).thenReturn(capabilityType);

    when(ref2.context()).thenReturn(mock(CapabilityContext.class));
    when(ref2.context().type()).thenReturn(capabilityType);

    when(ref3.context()).thenReturn(mock(CapabilityContext.class));
    when(ref3.context().type()).thenReturn(capabilityType(Capability.class.getName()));

    final CapabilityDescriptorRegistry descriptorRegistry = mock(CapabilityDescriptorRegistry.class);
    final CapabilityDescriptor descriptor = mock(CapabilityDescriptor.class);

    when(descriptor.name()).thenReturn(this.getClass().getSimpleName());
    when(descriptorRegistry.get(capabilityType)).thenReturn(descriptor);

    underTest = new CapabilityOfTypeActiveCondition(
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
   * Condition should not be satisfied if capability of specified type exists but is not active.
   */
  @Test
  public void capabilityOfTypeActive01() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(false);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(false));
  }

  /**
   * Condition should be satisfied if capability of specified type exists and is active.
   */
  @Test
  public void capabilityOfTypeActive02() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should not be re-satisfied if a new active capability of specified type is added.
   */
  @Test
  public void capabilityOfTypeActive03() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    when(ref2.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref2));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should remain satisfied if another active capability of the specified type is removed.
   */
  @Test
  public void capabilityOfTypeActive04() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    when(ref2.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref2));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref2)).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.AfterRemove(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should remain satisfied if another active capability of the specified type is passivated.
   */
  @Test
  public void capabilityOfTypeActive05() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    when(ref2.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref2));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref2)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.BeforePassivated(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest));
  }

  /**
   * Condition should become unsatisfied when all capabilities have been removed.
   */
  @Test
  public void capabilityOfTypeActive06() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Collections.emptyList()).when(capabilityRegistry).getAll();
    underTest.handle(new CapabilityEvent.AfterRemove(ref1));
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
  }

  /**
   * Condition should remain satisfied if a condition of a different type is passivated.
   */
  @Test
  public void capabilityOfTypeActive07() {
    doReturn(Arrays.asList(ref1)).when(capabilityRegistry).getAll();
    when(ref1.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref1));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref3)).when(capabilityRegistry).getAll();
    when(ref3.context().isActive()).thenReturn(true);
    underTest.handle(new CapabilityEvent.Created(ref3));
    assertThat(underTest.isSatisfied(), is(true));

    doReturn(Arrays.asList(ref1, ref3)).when(capabilityRegistry).getAll();
    when(ref3.context().isActive()).thenReturn(false);
    underTest.handle(new CapabilityEvent.BeforePassivated(ref3));
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
