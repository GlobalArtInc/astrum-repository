
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
  public void sessionFilterChain_hasAsAuthcBeforeAsApikeyAuthc() {
    FilterChain chain = new HttpBridgeConfiguration().httpBridgeFilterChain();
    assertFilterOrder(chain);
  }

  @Test
  public void jwtFilterChain_hasAsAuthcBeforeAsApikeyAuthc() {
    FilterChain chain = new HttpBridgeConfiguration().httpBridgeFilterChain_jwt();
    assertFilterOrder(chain);
  }

  private static void assertFilterOrder(final FilterChain chain) {
    String expression = chain.getFilterExpression();
    int authcPos = expression.indexOf(AstrumAuthenticationFilter.NAME);
    int apikeyPos = expression.indexOf(ApiKeyAuthenticationFilter.NAME);

    assertThat("as-authc must appear in the filter expression", authcPos, is(not(-1)));
    assertThat("as-apikey-authc must appear in the filter expression", apikeyPos, is(not(-1)));
    assertThat("as-authc must precede as-apikey-authc — reordering breaks service account auth",
        authcPos, is(not(greaterThan(apikeyPos))));
  }
}
