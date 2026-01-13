
package org.globalart.astrum.capability.condition.internal;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.condition.EventManagerTestSupport;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.ExpectedException;
import org.mockito.InOrder;
import org.mockito.Mock;
import org.mockito.Mockito;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.capability.CapabilityIdentity.capabilityIdentity;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

/**
 * {@link CapabilityHasNoDuplicatesCondition} UTs.
 */
public class CapabilityHasNoDuplicatesConditionTest
    extends EventManagerTestSupport
{

  @Rule
  public ExpectedException thrown = ExpectedException.none();

  @Mock
  private CapabilityRegistry capabilityRegistry;

  private CapabilityHasNoDuplicatesCondition underTest;

  private CapabilityReference reference;

  private CapabilityContext context;

  private CapabilityDescriptor descriptor;

  @Before
  public void setUpCondition() throws Exception {
    reference = createReference("testRef1", "testType");

    context = reference.context();
    descriptor = context.descriptor();

    underTest = new CapabilityHasNoDuplicatesCondition(eventManager);
  }

  @Test
  public void standardLifecycle() {
    assertThat(underTest.isSatisfied(), is(false));

    underTest.setContext(reference.context());

    assertThat(underTest.isSatisfied(), is(false));

    underTest.bind();

    assertThat(underTest.isSatisfied(), is(true));

    underTest.release();

    InOrder inOrder = Mockito.inOrder(eventManager);
    inOrder.verify(eventManager).register(underTest);
    inOrder.verify(eventManager).unregister(underTest);
    inOrder.verifyNoMoreInteractions();
  }

  @Test
  public void duplicatesDetectedDuringBind() {
    underTest.setContext(reference.context());

    when(descriptor.isDuplicated(context.id(), context.properties())).thenReturn(true);

    underTest.bind();

    assertThat(underTest.isSatisfied(), is(false));

    underTest.release();
  }

  @Test
  public void duplicatesDetectedDuringEvents() {
    CapabilityReference unrelatedRef = createReference("testRef2", "anotherType");
    CapabilityReference duplicateRef = createReference("testRef3", "testType");

    underTest.setContext(reference.context());
    underTest.bind();

    // only checked after matching event
    when(descriptor.isDuplicated(context.id(), context.properties())).thenReturn(true);
    assertThat(underTest.isSatisfied(), is(true));
    // different type, shouldn't trigger change
    underTest.handle(new CapabilityEvent.Created(unrelatedRef));
    assertThat(underTest.isSatisfied(), is(true));

    // same type, condition should check for dups
    underTest.handle(new CapabilityEvent.Created(duplicateRef));

    assertThat(underTest.isSatisfied(), is(false));

    // only checked after matching event
    when(descriptor.isDuplicated(context.id(), context.properties())).thenReturn(false);
    assertThat(underTest.isSatisfied(), is(false));
    // different type, shouldn't trigger change
    underTest.handle(new CapabilityEvent.AfterRemove(unrelatedRef));
    assertThat(underTest.isSatisfied(), is(false));

    // same type, condition should check for dups
    underTest.handle(new CapabilityEvent.AfterRemove(duplicateRef));

    assertThat(underTest.isSatisfied(), is(true));

    underTest.release();

    InOrder inOrder = Mockito.inOrder(descriptor);
    // isDuplicated should only be called during bind + once for each event of same type
    inOrder.verify(descriptor, times(3)).isDuplicated(context.id(), context.properties());
    inOrder.verifyNoMoreInteractions();
  }

  /**
   * Verify that contextualization fails if already bound.
   */
  @Test
  public void contextualizationFailsWhenAlreadyBounded() {
    thrown.expect(IllegalStateException.class);
    thrown.expectMessage("Cannot contextualize when already bound");
    underTest.setContext(reference.context());
    underTest.bind();
    underTest.setContext(reference.context());
  }

  /**
   * Verify that contextualization fails if already contextualized.
   */
  @Test
  public void contextualizationFailsWhenAlreadyContextualized() {
    thrown.expect(IllegalStateException.class);
    thrown.expectMessage("Already contextualized");
    underTest.setContext(reference.context());
    underTest.setContext(reference.context());
  }

  private static CapabilityReference createReference(final String id, final String type) {
    CapabilityDescriptor descriptor = mock(CapabilityDescriptor.class);

    CapabilityContext context = mock(CapabilityContext.class);
    when(context.id()).thenReturn(capabilityIdentity(id));
    Map<String, String> testProperties = ImmutableMap.of("testKey", "testValue");
    when(context.properties()).thenReturn(testProperties);
    when(context.descriptor()).thenReturn(descriptor);
    when(context.type()).thenReturn(capabilityType(type));

    CapabilityReference reference = mock(CapabilityReference.class);
    when(reference.context()).thenReturn(context);

    return reference;
  }
}
