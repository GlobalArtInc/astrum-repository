
package org.globalart.astrum.repository.routing.internal;

import java.util.Collection;
import java.util.List;
import java.util.stream.IntStream;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.routing.RoutingMode;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.routing.RoutingMode.ALLOW;
import static org.globalart.astrum.repository.routing.RoutingMode.BLOCK;

public class RoutingRuleDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(RoutingRuleDAO.class);

  private DataSession<?> session;

  private RoutingRuleDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(RoutingRuleDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testCRUD() {
    RoutingRuleData routingRule = routingRule("foo", ALLOW, "desc", "a", "b", "c");
    // the routing rule is stored
    dao.create(routingRule);
    // it is read back
    RoutingRuleData read = dao.read(routingRule.getId()).orElse(null);
    // the read value matches the original
    assertThat(read.getId(), is(routingRule.getId()));
    assertThat(read.getName(), is(routingRule.getName()));
    assertThat(read.mode(), is(routingRule.mode()));
    assertThat(read.description(), is(routingRule.description()));
    assertThat(read.matchers(), is(routingRule.matchers()));

    // the routing rule is found by name
    read = dao.readByName(routingRule.getName()).get();
    // the found value matches the original
    assertThat(read.getName(), is(routingRule.getName()));
    assertThat(read.mode(), is(routingRule.mode()));
    assertThat(read.description(), is(routingRule.description()));
    assertThat(read.matchers(), is(routingRule.matchers()));

    // it is updated
    routingRule.name("foo2");
    routingRule.mode(BLOCK);
    routingRule.description("desc2");
    routingRule.matchers(List.of("x", "y", "z"));
    dao.update(routingRule);
    // it is read back
    RoutingRuleData update = dao.read(routingRule.getId()).orElse(null);
    // the read value matches the update
    assertThat(update.getName(), is(routingRule.getName()));
    assertThat(update.mode(), is(routingRule.mode()));
    assertThat(update.description(), is(routingRule.description()));
    assertThat(update.matchers(), is(routingRule.matchers()));

    // the updated routing rule is found by name
    read = dao.readByName(routingRule.getName()).get();
    // the found value matches the original
    assertThat(read.getName(), is(routingRule.getName()));
    assertThat(read.mode(), is(routingRule.mode()));
    assertThat(read.description(), is(routingRule.description()));
    assertThat(read.matchers(), is(routingRule.matchers()));

    // it is deleted
    dao.delete(routingRule.getId());
    // no routingRule is found by that name
    assertFalse(dao.read(routingRule.getId()).isPresent());
    // several rules are created
    IntStream.range(1, 6)
        .forEach(it -> dao.create(new RoutingRuleData().name("foo" + it).mode(ALLOW).matchers(List.of("a"))));
    // browsing finds them all
    Collection<RoutingRuleData> items = (Collection<RoutingRuleData>) dao.browse();
    assertThat(items, hasSize(5));

    // a rule with the same name is created
    dao.create(routingRule("food", ALLOW).matchers(List.of("a")));
    // a exception is thrown
    assertThrows(Exception.class, () -> dao.create(routingRule("food", ALLOW).matchers(List.of("a"))));
  }

  private static RoutingRuleData routingRule(final String name, final RoutingMode mode) {
    return new RoutingRuleData().name(name).mode(mode);
  }

  private static RoutingRuleData routingRule(
      final String name,
      final RoutingMode mode,
      final String description,
      final String... matchers)
  {
    return new RoutingRuleData().name(name).mode(mode).description(description).matchers(List.of(matchers));
  }

}
