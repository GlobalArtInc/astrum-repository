
package org.globalart.astrum.repository.routing.internal;

import java.util.UUID;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.routing.RoutingRule;
import org.globalart.astrum.repository.routing.RoutingRuleInvalidatedEvent;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class RoutingRuleCacheTest
{
  @Mock
  private RoutingRuleStore routingRuleStore;

  @Mock
  private Repository repository;

  @Mock
  private Configuration configuration;

  private RoutingRuleCache underTest;

  @Before
  public void setup() {
    underTest = new RoutingRuleCache(routingRuleStore);
    when(repository.getConfiguration()).thenReturn(configuration);
  }

  @Test
  public void testGetRoutingRule_returnsRuleFromStore() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    RoutingRule rule = mock(RoutingRule.class);
    when(configuration.getRoutingRuleId()).thenReturn(ruleId);
    when(routingRuleStore.getById(ruleId.getValue())).thenReturn(rule);

    assertThat(underTest.getRoutingRule(repository), is(rule));
  }

  @Test
  public void testGetRoutingRule_returnsNullWhenNoRuleAssigned() {
    when(configuration.getRoutingRuleId()).thenReturn(null);

    assertThat(underTest.getRoutingRule(repository), is(nullValue()));
  }

  @Test
  public void testGetRoutingRuleId_returnsIdFromConfiguration() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    when(configuration.getRoutingRuleId()).thenReturn(ruleId);

    assertThat(underTest.getRoutingRuleId(repository), is(ruleId));
  }

  @Test
  public void testGetRoutingRuleId_returnsNullWhenNoRuleAssigned() {
    when(configuration.getRoutingRuleId()).thenReturn(null);

    assertThat(underTest.getRoutingRuleId(repository), is(nullValue()));
  }

  // Core fix: when RoutingRuleInvalidatedEvent arrives (from any HA node),
  // the rule cache must be invalidated so the next request reloads from the store
  @Test
  public void testHandle_routingRuleInvalidatedEvent_invalidatesRuleCache() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    RoutingRule rule = mock(RoutingRule.class);
    when(configuration.getRoutingRuleId()).thenReturn(ruleId);
    when(routingRuleStore.getById(ruleId.getValue())).thenReturn(rule);

    // warm up cache
    underTest.getRoutingRule(repository);
    verify(routingRuleStore, times(1)).getById(ruleId.getValue());

    // event arrives (e.g. fired by another HA node)
    underTest.handle(new RoutingRuleInvalidatedEvent(ruleId));

    // next request must reload from store, not serve stale cached value
    underTest.getRoutingRule(repository);
    verify(routingRuleStore, times(2)).getById(ruleId.getValue());
  }

  @Test
  public void testHandle_repositoryDeletedEvent_invalidatesRepositoryCache() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    when(configuration.getRoutingRuleId()).thenReturn(ruleId);
    when(routingRuleStore.getById(ruleId.getValue())).thenReturn(mock(RoutingRule.class));

    // warm up cache
    underTest.getRoutingRule(repository);
    verify(configuration, times(1)).getRoutingRuleId();

    underTest.handle(new RepositoryDeletedEvent(repository));

    // repo→rule mapping must be reloaded
    underTest.getRoutingRule(repository);
    verify(configuration, times(2)).getRoutingRuleId();
  }

  @Test
  public void testHandle_repositoryUpdatedEvent_invalidatesRepositoryCache() {
    EntityId ruleId = new EntityUUID(UUID.randomUUID());
    when(configuration.getRoutingRuleId()).thenReturn(ruleId);
    when(routingRuleStore.getById(ruleId.getValue())).thenReturn(mock(RoutingRule.class));

    // warm up cache
    underTest.getRoutingRule(repository);
    verify(configuration, times(1)).getRoutingRuleId();

    underTest.handle(new RepositoryUpdatedEvent(repository, mock(Configuration.class)));

    // repo→rule mapping must be reloaded after config change
    underTest.getRoutingRule(repository);
    verify(configuration, times(2)).getRoutingRuleId();
  }
}
