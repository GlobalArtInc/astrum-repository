
package org.globalart.astrum.coreui.internal.welcome;

import java.util.Map;

import org.globalart.astrum.common.node.NodeAccess;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.coreui.internal.welcome.WelcomeStateContributor.NODE_ID;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class WelcomeStateContributorTest
{
  @Mock
  private NodeAccess nodeAccess;

  private WelcomeStateContributor underTest;

  @Before
  public void setUp() {
    when(nodeAccess.getId()).thenReturn("test-node-id");
  }

  @Test
  public void shouldReturnFeatureFlagEnabled() {
    underTest = new WelcomeStateContributor(true, nodeAccess);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum.react.welcome"), is(true));
  }

  @Test
  public void shouldReturnFeatureFlagDisabled() {
    underTest = new WelcomeStateContributor(false, nodeAccess);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum.react.welcome"), is(false));
  }

  @Test
  public void shouldReturnNodeId() {
    underTest = new WelcomeStateContributor(true, nodeAccess);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(NODE_ID), is("test-node-id"));
  }
}
