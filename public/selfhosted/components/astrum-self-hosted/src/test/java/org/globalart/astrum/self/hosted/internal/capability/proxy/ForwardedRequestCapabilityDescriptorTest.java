
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class ForwardedRequestCapabilityDescriptorTest
{
  @Test
  void testType() {
    ForwardedRequestCapabilityDescriptor undertest = new ForwardedRequestCapabilityDescriptor();

    assertThat(undertest.type().toString(), is("http.forwarded"));
  }

  @Test
  void testName() {
    ForwardedRequestCapabilityDescriptor undertest = new ForwardedRequestCapabilityDescriptor();

    assertThat(undertest.name(), is("HTTP Forwarded Headers"));
  }
}
