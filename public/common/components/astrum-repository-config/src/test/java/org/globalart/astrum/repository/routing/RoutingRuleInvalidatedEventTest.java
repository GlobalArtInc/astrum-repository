
package org.globalart.astrum.repository.routing;

import java.util.UUID;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class RoutingRuleInvalidatedEventTest
{
  // Core fix: event must extend EventWithSource to be picked up by DistributedEventPublisher
  @Test
  public void testExtendsEventWithSource() {
    assertTrue(EventWithSource.class.isAssignableFrom(RoutingRuleInvalidatedEvent.class));
  }

  // Required by Jackson for deserialization on remote nodes
  @Test
  public void testNoArgConstructorExists() throws Exception {
    RoutingRuleInvalidatedEvent event = RoutingRuleInvalidatedEvent.class.getDeclaredConstructor().newInstance();
    assertNotNull(event);
  }

  @Test
  public void testRoutingRuleIdPreserved() {
    UUID uuid = UUID.randomUUID();
    EntityId id = new EntityUUID(uuid);

    RoutingRuleInvalidatedEvent event = new RoutingRuleInvalidatedEvent(id);

    assertThat(event.getRoutingRuleId(), is(uuid.toString()));
  }

  @Test
  public void testSetRoutingRuleId() {
    RoutingRuleInvalidatedEvent event = new RoutingRuleInvalidatedEvent();
    String id = UUID.randomUUID().toString();

    event.setRoutingRuleId(id);

    assertThat(event.getRoutingRuleId(), is(id));
  }

  // Verifies the event survives Jackson round-trip used by the distributed event pipeline
  @Test
  public void testJacksonSerializable() throws Exception {
    ObjectMapper mapper = new ObjectMapper();
    UUID uuid = UUID.randomUUID();
    EntityId id = new EntityUUID(uuid);

    RoutingRuleInvalidatedEvent original = new RoutingRuleInvalidatedEvent(id);
    String json = mapper.writeValueAsString(original);
    RoutingRuleInvalidatedEvent restored = mapper.readValue(json, RoutingRuleInvalidatedEvent.class);

    assertThat(restored.getRoutingRuleId(), is(uuid.toString()));
  }
}
