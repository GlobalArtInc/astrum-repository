
package org.globalart.astrum.rest;

import com.google.common.collect.ImmutableList;
import org.junit.jupiter.api.Test;

import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class WebApplicationMessageExceptionTest
{

  /**
   * Method under test:
   * {@link WebApplicationMessageException#WebApplicationMessageException(Response.Status, Object, String)}
   */
  @Test
  public void testConstructor() {
    WebApplicationMessageException exception = new WebApplicationMessageException(
        Response.Status.BAD_REQUEST, "Message", MediaType.APPLICATION_JSON);
    Response response = exception.getResponse();

    assertEquals(400, response.getStatus());

    Object entity = response.getEntity();

    assertTrue(entity instanceof ValidationErrorXO);
    assertEquals("Message", ((ValidationErrorXO) entity).getMessage());
    assertEquals(ValidationErrorXO.GENERIC, ((ValidationErrorXO) entity).getId());
    assertEquals(ImmutableList.of(MediaType.APPLICATION_JSON), response.getHeaders().get("Content-Type"));
  }

  /**
   * Method under test: {@link WebApplicationMessageException#WebApplicationMessageException(Response.Status, String)}
   */
  @Test
  public void testConstructorNoMediaType() {
    WebApplicationMessageException exception = new WebApplicationMessageException(
        Response.Status.NOT_FOUND, "Message");
    Response response = exception.getResponse();

    assertEquals(404, response.getStatus());

    Object entity = response.getEntity();

    assertTrue(entity instanceof ValidationErrorXO);
    assertEquals("Message", ((ValidationErrorXO) entity).getMessage());
    assertEquals(ValidationErrorXO.GENERIC, ((ValidationErrorXO) entity).getId());
    assertEquals(ImmutableList.of(MediaType.TEXT_PLAIN), response.getHeaders().get("Content-Type"));
  }
}
