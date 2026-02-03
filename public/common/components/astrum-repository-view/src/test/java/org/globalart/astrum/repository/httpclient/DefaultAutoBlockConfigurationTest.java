
package org.globalart.astrum.repository.httpclient;

import org.junit.Before;
import org.junit.Test;

import static org.apache.http.HttpStatus.SC_UNAUTHORIZED;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.repository.http.HttpStatus.PROXY_AUTHENTICATION_REQUIRED;

public class DefaultAutoBlockConfigurationTest
{
  private DefaultAutoBlockConfiguration underTest;

  @Before
  public void setup() throws Exception {
    underTest = new DefaultAutoBlockConfiguration();
  }

  @Test
  public void blockUnauthorized() {
    assertTrue(underTest.shouldBlock(SC_UNAUTHORIZED));
  }

  @Test
  public void blockProxyAuthenticationRequired() {
    assertTrue(underTest.shouldBlock(PROXY_AUTHENTICATION_REQUIRED));
  }

  @Test
  public void blockAll500Statuses() {
    for (int status = 500; status <= 599; status++) {
      assertTrue(underTest.shouldBlock(status));
    }
  }
}
