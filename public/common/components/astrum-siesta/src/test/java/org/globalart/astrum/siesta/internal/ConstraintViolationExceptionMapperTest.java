
package org.globalart.astrum.siesta.internal;

import java.util.Collections;
import java.util.List;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.ElementKind;
import jakarta.validation.Path;
import jakarta.validation.Path.Node;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.ValidationErrorXO;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class ConstraintViolationExceptionMapperTest
{
  private ConstraintViolationExceptionMapper mapper;

  @BeforeEach
  void setup() {
    mapper = new ConstraintViolationExceptionMapper();
  }

  @Test
  void testGetValidationErrors() {
    ConstraintViolation<?> violation = mock(ConstraintViolation.class);
    when(violation.getLeafBean()).thenReturn(new Object());
    when(violation.getPropertyPath()).thenReturn(mock(Path.class));
    when(violation.getMessage()).thenReturn("Test message");

    Set<ConstraintViolation<?>> violations = Collections.singleton(violation);
    ConstraintViolationException exception = new ConstraintViolationException(violations);

    List<ValidationErrorXO> errors = mapper.getValidationErrors(exception);

    assertThat(errors, hasSize(1));
    assertThat(errors.get(0).getMessage(), equalTo("Test message"));
  }

  @Test
  void testGetStatusInternalServerError() {
    ConstraintViolation<?> violation = mock(ConstraintViolation.class);

    Path path = mock(Path.class);
    Node nodePath = mock(Node.class);
    when(nodePath.getKind()).thenReturn(ElementKind.RETURN_VALUE);
    when(path.iterator()).thenReturn(Collections.singleton(nodePath).iterator());
    when(violation.getPropertyPath()).thenReturn(path);

    Set<ConstraintViolation<?>> violations = Collections.singleton(violation);
    ConstraintViolationException exception = new ConstraintViolationException(violations);

    Status status = mapper.getStatus(exception);

    assertThat(status, equalTo(Status.INTERNAL_SERVER_ERROR));
  }

  @Test
  void testGetStatusBadRequest() {
    ConstraintViolation<?> violation = mock(ConstraintViolation.class);
    Path path = mock(Path.class);
    when(path.iterator()).thenReturn(Collections.singleton(mock(Path.Node.class)).iterator());
    when(violation.getPropertyPath()).thenReturn(path);

    Set<ConstraintViolation<?>> violations = Collections.singleton(violation);
    ConstraintViolationException exception = new ConstraintViolationException(violations);

    Status status = mapper.getStatus(exception);

    assertThat(status, equalTo(Status.BAD_REQUEST));
  }

  @Test
  void testGetStatusBadRequestEmptyViolations() {
    Set<ConstraintViolation<?>> violations = Collections.emptySet();
    ConstraintViolationException exception = new ConstraintViolationException(violations);

    Status status = mapper.getStatus(exception);

    assertThat(status, equalTo(Status.BAD_REQUEST));
  }
}
