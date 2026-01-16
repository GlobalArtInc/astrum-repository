
package org.globalart.astrum.siesta.internal;

import java.util.List;

import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.ValidationErrorXO;

import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.JsonMappingException.Reference;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;

class JsonMappingExceptionMapperTest
{
  private JsonMappingExceptionMapper mapper;

  @BeforeEach
  void setup() {
    mapper = new JsonMappingExceptionMapper();
  }

  @Test
  void testConvert() {
    Reference reference = new Reference(null, "testField");
    JsonMappingException exception = new JsonMappingException("test json mapping exception");
    exception.prependPath(reference);

    try (Response response = mapper.convert(exception, "testId")) {
      assertThat(response.getStatus(), equalTo(Response.Status.BAD_REQUEST.getStatusCode()));
      List<ValidationErrorXO> errors = (List<ValidationErrorXO>) response.getEntity();
      assertThat(errors, hasSize(1));
      ValidationErrorXO error = errors.get(0);
      assertThat(error.getId(), equalTo("testField"));
      assertThat(error.getMessage(), equalTo("test json mapping exception"));
    }
  }
}
