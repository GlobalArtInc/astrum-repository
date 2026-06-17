
package org.globalart.astrum.coreui.internal.http;

import java.util.Map;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.httpclient.HttpDefaultsCustomizer;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class HttpStateContributorTest
{
  @Mock
  private HttpDefaultsCustomizer customizer;

  private HttpStateContributor underTest;

  @Before
  public void setUp() {
    when(customizer.getRequestTimeout()).thenReturn(Time.seconds(30));
    when(customizer.getRetryCount()).thenReturn(2);
    underTest = new HttpStateContributor(true, "/", customizer);
  }

  @Test
  public void shouldReturnFeatureFlagEnabled() {
    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum.react.httpSettings"), is(true));
  }

  @Test
  public void shouldReturnFeatureFlagDisabled() {
    underTest = new HttpStateContributor(false, "/", customizer);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum.react.httpSettings"), is(false));
  }

  @Test
  public void shouldReturnRequestTimeout() {
    Map<String, Object> state = underTest.getState();

    assertThat(state.get("requestTimeout"), is(Time.seconds(30)));
  }

  @Test
  public void shouldReturnRetryCount() {
    Map<String, Object> state = underTest.getState();

    assertThat(state.get("retryCount"), is(2));
  }

  @Test
  public void shouldReturnContextPath() {
    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum-context-path"), is("/"));
  }

  @Test
  public void shouldReturnCustomContextPath() {
    underTest = new HttpStateContributor(true, "/astrum", customizer);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("astrum-context-path"), is("/astrum"));
  }
}
