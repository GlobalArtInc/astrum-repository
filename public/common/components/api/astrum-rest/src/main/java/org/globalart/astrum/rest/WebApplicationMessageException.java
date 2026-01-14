
package org.globalart.astrum.rest;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.GenericEntity;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.TEXT_PLAIN;

/**
 * {@link WebApplicationException} with {@link Status} and a text message.
 *
 * @since 3.8
 */
public class WebApplicationMessageException
    extends WebApplicationException
{
  public WebApplicationMessageException(final Status status, final String message) {
    this(status, message, TEXT_PLAIN);
  }

  public WebApplicationMessageException(final Status status, final Object message, final String mediaType) {
    super(Response.status(checkNotNull(status))
        .entity(new GenericEntity<>(new ValidationErrorXO(checkNotNull(message).toString()), ValidationErrorXO.class))
        .type(mediaType)
        .build());
  }

  public WebApplicationMessageException(
      final Status status,
      final String id,
      final Object message,
      final String mediaType)
  {
    super(Response.status(checkNotNull(status))
        .entity(
            new GenericEntity<>(new ValidationErrorXO(id, checkNotNull(message).toString()), ValidationErrorXO.class))
        .type(mediaType)
        .build());
  }

  public WebApplicationMessageException(int status, final Object message, final String mediaType) {
    super(Response.status(status)
        .entity(new GenericEntity<>(new ValidationErrorXO(checkNotNull(message).toString()), ValidationErrorXO.class))
        .type(mediaType)
        .build());
  }

  public WebApplicationMessageException(int status, final String id, final Object message, final String mediaType) {
    super(Response.status(status)
        .entity(
            new GenericEntity<>(new ValidationErrorXO(id, checkNotNull(message).toString()), ValidationErrorXO.class))
        .type(mediaType)
        .build());
  }
}
