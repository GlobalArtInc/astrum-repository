
package org.globalart.astrum.rest;

import org.junit.Test;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import static jakarta.ws.rs.core.Response.Status.*;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.*;

public class SimpleApiResponseTest
{

  @Test
  public void testOkResponseWithoutData() {
    Response simpleApiResponse = SimpleApiResponse.ok("message");
    assertResponse(simpleApiResponse, OK, null);
  }

  @Test
  public void testOkResponseWithData() {
    Response simpleApiResponse = SimpleApiResponse.ok("message", new Data("bar"));
    assertResponse(simpleApiResponse, OK, "bar");
  }

  @Test
  public void testNotFoundResponseWithoutData() {
    Response simpleApiResponse = SimpleApiResponse.notFound("message");
    assertResponse(simpleApiResponse, NOT_FOUND, null);
  }

  @Test
  public void testNotFoundResponseWithData() {
    Response simpleApiResponse = SimpleApiResponse.notFound("message", new Data("bar"));
    assertResponse(simpleApiResponse, NOT_FOUND, "bar");
  }

  @Test
  public void testBadRequestResponseWithoutData() {
    Response simpleApiResponse = SimpleApiResponse.badRequest("message");
    assertResponse(simpleApiResponse, BAD_REQUEST, null);
  }

  @Test
  public void testBadRequestResponseWithData() {
    Response simpleApiResponse = SimpleApiResponse.badRequest("message", new Data("bar"));
    assertResponse(simpleApiResponse, BAD_REQUEST, "bar");
  }

  @Test
  public void testUnauthorizedResponseWithoutData() {
    Response simpleApiResponse = SimpleApiResponse.unauthorized("message");
    assertResponse(simpleApiResponse, UNAUTHORIZED, null);
  }

  @Test
  public void testUnauthorizedResponseWithData() {
    Response simpleApiResponse = SimpleApiResponse.unauthorized("message", new Data("bar"));
    assertResponse(simpleApiResponse, UNAUTHORIZED, "bar");
  }

  private void assertResponse(Response simpleApiResponse, Status status, String value) {
    assertThat(simpleApiResponse.getStatus(), is(status.getStatusCode()));
    SimpleApiResponse entity = (SimpleApiResponse) simpleApiResponse.getEntity();
    assertThat(entity.getStatus(), is(status.getStatusCode()));
    assertThat(entity.getMessage(), is("message"));
    if (value == null) {
      assertThat(entity.getData(), is(nullValue()));
    }
    else {
      assertThat(((Data) entity.getData()).getFoo(), is("bar"));
    }
  }

  private static class Data
  {
    private final String foo;

    public Data(String foo) {
      this.foo = foo;
    }

    public String getFoo() {
      return foo;
    }
  }
}
