
package org.globalart.astrum.siesta.internal;

import jakarta.ws.rs.core.Response;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;

class UnexpectedExceptionMapperTest
{
  private UnexpectedExceptionMapper mapper;

  @BeforeEach
  void setup() {
    mapper = new UnexpectedExceptionMapper();
  }

  @Test
  void testConvert() {
    Throwable exception = new RuntimeException("Unexpected error");
    try (Response response = mapper.convert(exception, "testId")) {
      assertThat(response.getStatus(), is(Response.Status.INTERNAL_SERVER_ERROR.getStatusCode()));
      assertThat(response.getEntity().toString(), containsString("ERROR: (ID testId)"));
    }
  }
}
