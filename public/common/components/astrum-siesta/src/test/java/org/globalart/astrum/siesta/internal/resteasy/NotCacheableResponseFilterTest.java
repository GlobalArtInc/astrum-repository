
package org.globalart.astrum.siesta.internal.resteasy;

import java.io.IOException;

import jakarta.ws.rs.container.ContainerRequestContext;
import jakarta.ws.rs.container.ContainerResponseContext;
import jakarta.ws.rs.core.MultivaluedHashMap;
import jakarta.ws.rs.core.MultivaluedMap;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static jakarta.ws.rs.core.HttpHeaders.CACHE_CONTROL;
import static jakarta.ws.rs.core.HttpHeaders.EXPIRES;
import static org.hamcrest.CoreMatchers.hasItem;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class NotCacheableResponseFilterTest
{
  private NotCacheableResponseFilter underTest;

  @Mock
  private ContainerRequestContext requestContext;

  @Mock
  private ContainerResponseContext responseContext;

  @BeforeEach
  void setup() {
    underTest = new NotCacheableResponseFilter();
  }

  @Test
  void testFilter() throws IOException {
    MultivaluedMap<String, Object> headers = spy(new MultivaluedHashMap<>());
    when(responseContext.getHeaders()).thenReturn(headers);

    underTest.filter(requestContext, responseContext);

    assertThat(headers.size(), is(3));
    assertThat(headers.get(CACHE_CONTROL),
        hasItem("no-cache, must-revalidate, no-transform, no-store, proxy-revalidate, s-maxage=0, max-age=0"));
    assertThat(headers.get(EXPIRES), hasItem(0));
    assertThat(headers.get("Pragma"), hasItem("no-cache"));
  }
}
