
package org.globalart.astrum.siesta;

import java.util.List;

import jakarta.ws.rs.client.WebTarget;

import org.jboss.resteasy.client.jaxrs.ResteasyWebTarget;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Test use of {@link Echo} proxy to access {@link EchoResource}.
 */
class EchoIT
    extends SiestaTestSupport
{
  @Test
  void basic() {
    WebTarget target = client().target(url());
    Echo echo = ((ResteasyWebTarget) target).proxy(Echo.class);
    List<String> result = echo.get("hi");
    assertThat(result, notNullValue());
    assertThat(result, hasItem("foo=hi"));
  }
}
