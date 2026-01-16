
package org.globalart.astrum.siesta.internal;

import java.util.Map;

import org.globalart.astrum.rest.ExceptionMapperSupport;
import org.globalart.astrum.rest.ValidationErrorXO;

import com.google.common.net.HttpHeaders;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.GenericEntity;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.aMapWithSize;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.siesta.internal.WebappExceptionMapper.SIESTA_FAULTID;
import static org.globalart.astrum.siesta.internal.WebappExceptionMapper.STATUS_CODE;
import static org.globalart.astrum.siesta.internal.WebappExceptionMapper.STATUS_MESSAGE;

@ExtendWith(MockitoExtension.class)
class WebappExceptionMapperTest
{
  @Mock
  WebApplicationException exception;

  private WebappExceptionMapper mapper;

  @BeforeEach
  void setUp() {
    mapper = new WebappExceptionMapper();
  }

  @Test
  void testConvert() {
    Response remote = Response.status(Status.PAYMENT_REQUIRED)
        .entity(new GenericEntity<>(new ValidationErrorXO("testWebExMapper"), ValidationErrorXO.class))
        .build();
    when(exception.getResponse()).thenReturn(remote);

    try (Response result = mapper.convert(exception, "testId")) {
      assertThat(result.getStatus(), is(Status.PAYMENT_REQUIRED.getStatusCode()));
      assertThat(result.getHeaderString(HttpHeaders.CONTENT_TYPE), is(MediaType.APPLICATION_JSON));
      assertThat(result.getHeaderString(ExceptionMapperSupport.X_SIESTA_FAULT_ID), is("testId"));

      Map<String, Object> entity = (Map<String, Object>) result.getEntity();
      assertThat(entity, aMapWithSize(3));
      assertThat(entity, hasEntry(SIESTA_FAULTID, "testId"));
      assertThat(entity, hasEntry(STATUS_CODE, Status.PAYMENT_REQUIRED.getStatusCode()));
      assertThat(entity, hasEntry(STATUS_MESSAGE, Status.PAYMENT_REQUIRED));
    }
  }
}
