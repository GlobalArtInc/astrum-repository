
package org.globalart.astrum.bootstrap.siesta;

import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

class SiestaConfigurationTest
{
  private final SiestaConfiguration underTest = new SiestaConfiguration();

  @Test
  void testStatusWritableFilterChain_hasCorrectPath() {
    FilterChain chain = underTest.statusWritableFilterChain();

    assertThat(chain.getPathPattern(), is("/service/rest/v1/status/writable"));
  }

  @Test
  void testStatusWritableFilterChain_excludesAsAuthc() {
    FilterChain chain = underTest.statusWritableFilterChain();

    // as-authc triggers bcrypt + DB lookup on every probe — must not be in this chain
    assertThat(chain.getFilterExpression(), not(containsString(AstrumAuthenticationFilter.NAME)));
  }

  @Test
  void testStatusWritableFilterChain_includesAnonymousAndAntiCsrf() {
    FilterChain chain = underTest.statusWritableFilterChain();

    assertThat(chain.getFilterExpression(), containsString(AnonymousFilter.NAME));
    assertThat(chain.getFilterExpression(), containsString(AntiCsrfFilter.NAME));
  }
}
