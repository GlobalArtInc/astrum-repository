
package com.globalart.astrum.ssl.plugin.internal;

import javax.net.ssl.SSLContext;

import org.globalart.astrum.httpclient.SSLContextSelector;
import org.globalart.astrum.ssl.TrustStore;

import org.apache.http.protocol.BasicHttpContext;
import org.apache.http.protocol.HttpContext;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class HttpContextAttributeSSLContextSelectorTest
{
  @Mock
  private TrustStore trustStore;

  @Mock
  private SSLContext sslContext;

  private HttpContext httpContext;

  private HttpContextAttributeSSLContextSelector sslContextSelector;

  @Before
  public void setUp() {
    when(trustStore.getSSLContext()).thenReturn(sslContext);
    httpContext = new BasicHttpContext();
    sslContextSelector = new HttpContextAttributeSSLContextSelector(trustStore);
  }

  @Test
  public void testSelect_AttributeIsNull() {
    assertThat(sslContextSelector.select(httpContext), is(nullValue()));
  }

  @Test
  public void testSelect_AttributeIsFalse() {
    httpContext.setAttribute(SSLContextSelector.USE_TRUST_STORE, false);
    assertThat(sslContextSelector.select(httpContext), is(nullValue()));
  }

  @Test
  public void testSelect_AttributeIsTrue() {
    httpContext.setAttribute(SSLContextSelector.USE_TRUST_STORE, true);
    assertThat(sslContextSelector.select(httpContext), is(sslContext));
  }
}
