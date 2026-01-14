
package org.globalart.astrum.rest;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import com.fasterxml.jackson.annotation.JsonInclude;

import static com.fasterxml.jackson.annotation.JsonInclude.Include.NON_NULL;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static jakarta.ws.rs.core.Response.Status.OK;
import static jakarta.ws.rs.core.Response.Status.UNAUTHORIZED;

public class SimpleApiResponse
{

  private int status;

  private String message;

  @JsonInclude(NON_NULL)
  private Object data;

  public int getStatus() {
    return status;
  }

  public void setStatus(int status) {
    this.status = status;
  }

  public String getMessage() {
    return message;
  }

  public void setMessage(String message) {
    this.message = message;
  }

  public Object getData() {
    return data;
  }

  public void setData(Object data) {
    this.data = data;
  }

  public static Response ok(final String message) {
    return ok(message, null);
  }

  public static Response ok(final String message, final Object data) {
    return response(OK, message, data);
  }

  public static Response notFound(final String message) {
    return notFound(message, null);
  }

  public static Response notFound(final String message, final Object data) {
    return response(NOT_FOUND, message, data);
  }

  public static Response badRequest(final String message) {
    return badRequest(message, null);
  }

  public static Response badRequest(final String message, final Object data) {
    return response(BAD_REQUEST, message, data);
  }

  public static Response unauthorized(final String message) {
    return unauthorized(message, null);
  }

  public static Response unauthorized(final String message, final Object data) {
    return response(UNAUTHORIZED, message, data);
  }

  @Override
  public String toString() {
    return "SimpleApiResponse{" +
        "status=" + status +
        ", message='" + message + '\'' +
        ", data=" + data +
        '}';
  }

  private static Response response(final Status status, final String message, final Object data) {
    SimpleApiResponse response = new SimpleApiResponse();
    response.setStatus(status.getStatusCode());
    response.setMessage(message);
    response.setData(data);
    return Response.status(status).entity(response).type(APPLICATION_JSON).build();
  }
}
