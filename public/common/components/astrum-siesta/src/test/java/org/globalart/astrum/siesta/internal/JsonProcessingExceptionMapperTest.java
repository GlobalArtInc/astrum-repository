
package org.globalart.astrum.siesta.internal;

import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.ValidationErrorXO;

import com.fasterxml.jackson.core.JsonProcessingException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class JsonProcessingExceptionMapperTest
{
  private JsonProcessingExceptionMapper mapper;

  @BeforeEach
  void setup() {
    mapper = new JsonProcessingExceptionMapper();
  }

  @Test
  void testConvert() {
    JsonProcessingException exception = mock(JsonProcessingException.class);
    when(exception.getOriginalMessage()).thenReturn("test json processing exception");

    try (Response response = mapper.convert(exception, "testId")) {
      assertThat(response.getStatus(), equalTo(Response.Status.BAD_REQUEST.getStatusCode()));
      ValidationErrorXO error = (ValidationErrorXO) response.getEntity();
      assertThat(error.getId(), equalTo("*"));
      assertThat(error.getMessage(), equalTo("Could not process the input: test json processing exception"));
    }
  }
}
