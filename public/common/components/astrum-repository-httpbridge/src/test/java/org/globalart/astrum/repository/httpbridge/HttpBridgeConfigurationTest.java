
package org.globalart.astrum.repository.httpbridge;

import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyAuthenticationFilter;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.greaterThan;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

public class HttpBridgeConfigurationTest
{
  @Test
  public void sessionFilterChain_hasNxAuthcBeforeNxApikeyAuthc() {
    FilterChain chain = new HttpBridgeConfiguration().httpBridgeFilterChain();
    assertFilterOrder(chain);
  }

  @Test
  public void jwtFilterChain_hasNxAuthcBeforeNxApikeyAuthc() {
    FilterChain chain = new HttpBridgeConfiguration().httpBridgeFilterChain_jwt();
    assertFilterOrder(chain);
  }

  private static void assertFilterOrder(final FilterChain chain) {
    String expression = chain.getFilterExpression();
    int authcPos = expression.indexOf(AstrumAuthenticationFilter.NAME);
    int apikeyPos = expression.indexOf(ApiKeyAuthenticationFilter.NAME);

    assertThat("nx-authc must appear in the filter expression", authcPos, is(not(-1)));
    assertThat("nx-apikey-authc must appear in the filter expression", apikeyPos, is(not(-1)));
    assertThat("nx-authc must precede nx-apikey-authc — reordering breaks service account auth",
        authcPos, is(not(greaterThan(apikeyPos))));
  }
}
