
package org.globalart.astrum.repository.routing.internal;

import java.util.List;
import java.util.UUID;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.routing.RoutingMode;
import org.globalart.astrum.repository.routing.RoutingRuleInvalidatedEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertNotNull;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.verify;

@RunWith(MockitoJUnitRunner.class)
public class RoutingRuleStoreImplTest
{
  @Mock
  private DataSessionSupplier sessionSupplier;

  @Mock
  private EventManager eventManager;

  private RoutingRuleStoreImpl underTest;

  @Before
  public void setup() {
    // Subclass to bypass @Transactional DB operations — we only test event posting here
    underTest = new RoutingRuleStoreImpl(sessionSupplier, eventManager)
    {
      @Override
      protected void doUpdate(final RoutingRuleData rule) {
      }

      @Override
      protected void doDelete(final String name) {
      }
    };
  }

  @Test
  public void testUpdate_postsRoutingRuleInvalidatedEvent() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    RoutingRuleData rule = validRule(ruleId);

    underTest.update(rule);

    RoutingRuleInvalidatedEvent event = captureInvalidatedEvent();
    assertNotNull(event);
    assertThat(event.getRoutingRuleId(), is(ruleId.getValue()));
  }

  @Test
  public void testDelete_postsRoutingRuleInvalidatedEvent() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    RoutingRuleData rule = validRule(ruleId);

    underTest.delete(rule);

    RoutingRuleInvalidatedEvent event = captureInvalidatedEvent();
    assertNotNull(event);
    assertThat(event.getRoutingRuleId(), is(ruleId.getValue()));
  }

  // Regression guard: event must be a concrete class, not an anonymous inner class,
  // so Jackson can serialize it for the distributed event pipeline (ASTRUM-50997)
  @Test
  public void testUpdate_invalidatedEventIsConcreteClass() {
    underTest.update(validRule(new EntityUUID(UUID.randomUUID())));

    RoutingRuleInvalidatedEvent event = captureInvalidatedEvent();
    assertThat(event.getClass(), is(RoutingRuleInvalidatedEvent.class));
  }

  @Test
  public void testDelete_invalidatedEventIsConcreteClass() {
    underTest.delete(validRule(new EntityUUID(UUID.randomUUID())));

    RoutingRuleInvalidatedEvent event = captureInvalidatedEvent();
    assertThat(event.getClass(), is(RoutingRuleInvalidatedEvent.class));
  }

  private RoutingRuleInvalidatedEvent captureInvalidatedEvent() {
    ArgumentCaptor<Object> captor = ArgumentCaptor.forClass(Object.class);
    verify(eventManager, atLeastOnce()).post(captor.capture());
    return captor.getAllValues()
        .stream()
        .filter(e -> e instanceof RoutingRuleInvalidatedEvent)
        .map(e -> (RoutingRuleInvalidatedEvent) e)
        .findFirst()
        .orElse(null);
  }

  private static RoutingRuleData validRule(final EntityId id) {
    RoutingRuleData rule = new RoutingRuleData();
    rule.setId(id);
    rule.name("test-rule");
    rule.description("test description");
    rule.mode(RoutingMode.BLOCK);
    rule.matchers(List.of("/.*"));
    return rule;
  }
}
