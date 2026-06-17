
package org.globalart.astrum.coreui.internal;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests for {@link UiPluginDescriptorImpl}.
 */
public class UiPluginDescriptorImplTest
{
  private UiPluginDescriptorImpl underTest;

  @Before
  public void setUp() {
    underTest = new UiPluginDescriptorImpl();
  }

  @Test
  public void testGetPluginId() {
    assertThat(underTest.getPluginId(), is("astrum-coreui-plugin"));
  }

  @Test
  public void testGetNamespace() {
    assertThat(underTest.getNamespace(), is("NX.coreui"));
  }

  @Test
  public void testGetConfigClassName() {
    assertThat(underTest.getConfigClassName(), is("NX.coreui.app.PluginConfig"));
  }

  @Test
  public void testHasStyle() {
    assertThat(underTest.hasStyle(), is(true));
  }

  @Test
  public void testHasScript() {
    assertThat(underTest.hasScript(), is(true));
  }

  @Test
  public void testGetScripts_returnsEmptyList() {
    assertThat(underTest.getScripts(false), is(notNullValue()));
    assertThat(underTest.getScripts(false).isEmpty(), is(true));
    assertThat(underTest.getScripts(true), is(notNullValue()));
    assertThat(underTest.getScripts(true).isEmpty(), is(true));
  }
}
