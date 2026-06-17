
package org.globalart.astrum.coreui.internal;

import java.util.List;

import org.globalart.astrum.ui.UiUtil;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link CoreUiReactPluginDescriptorImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class CoreUiReactPluginDescriptorImplTest
{
  @Mock
  private UiUtil uiUtil;

  private CoreUiReactPluginDescriptorImpl underTest;

  @Before
  public void setUp() {
    when(uiUtil.getPathForFile("astrum-coreui-bundle.js")).thenReturn("/static/rapture/astrum-coreui-bundle.js");
    when(uiUtil.getPathForFile("astrum-coreui-bundle.debug.js"))
        .thenReturn("/static/rapture/astrum-coreui-bundle.debug.js");
    when(uiUtil.getPathForFile("astrum-coreui-bundle.css")).thenReturn("/static/rapture/astrum-coreui-bundle.css");

    underTest = new CoreUiReactPluginDescriptorImpl(uiUtil);
  }

  @Test
  public void testGetName() {
    assertThat(underTest.getName(), is("astrum-coreui-plugin"));
  }

  @Test
  public void testGetScripts_notDebug() {
    List<String> scripts = underTest.getScripts(false);
    assertThat(scripts, is(notNullValue()));
    assertThat(scripts, contains("/static/rapture/astrum-coreui-bundle.js"));
  }

  @Test
  public void testGetScripts_debug() {
    List<String> scripts = underTest.getScripts(true);
    assertThat(scripts, is(notNullValue()));
    assertThat(scripts, contains("/static/rapture/astrum-coreui-bundle.debug.js"));
  }

  @Test
  public void testGetStyles() {
    List<String> styles = underTest.getStyles();
    assertThat(styles, is(notNullValue()));
    assertThat(styles, contains("/static/rapture/astrum-coreui-bundle.css"));
  }

  @Test
  public void testGetScripts_debugFalseReturnsProdScripts() {
    List<String> debugScripts = underTest.getScripts(true);
    List<String> prodScripts = underTest.getScripts(false);

    assertThat(debugScripts.get(0).contains("debug"), is(true));
    assertThat(prodScripts.get(0).contains("debug"), is(false));
  }
}
