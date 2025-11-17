
package org.globalart.astrum.internal.node.datastore;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@RunWith(MockitoJUnitRunner.Silent.class)
public class NodeIdStoreImplTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(NodeIdDAO.class);

  @Mock
  private EventManager eventManager;

  private NodeIdStoreImpl underTest;

  @Before
  public void setup() {
    underTest = new NodeIdStoreImpl(sessionRule);
    underTest.setDependencies(eventManager);
  }

  @Test
  public void testGetOrCreate_existing() {
    underTest.set("foo");

    String result = underTest.getOrCreate();

    assertThat(result, is("foo"));

    assertThat(underTest.get().get(), is("foo"));
  }

  @Test
  public void testGetOrCreate_unset() {
    String result = underTest.getOrCreate();

    assertThat(underTest.get().isPresent(), is(true));
    assertThat(result, is(underTest.get().get()));
  }
}
