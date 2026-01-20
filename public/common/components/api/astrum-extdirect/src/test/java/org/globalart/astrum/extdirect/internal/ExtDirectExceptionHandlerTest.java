
package org.globalart.astrum.extdirect.internal;

import java.util.Collections;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.Path;

import org.apache.shiro.authz.UnauthorizedException;
import org.globalart.astrum.extdirect.model.ErrorResponse;
import org.globalart.astrum.extdirect.model.ValidationResponse;

import com.softwarementors.extjs.djn.api.RegisteredMethod;
import fake.com.globalart.insight.rm.rest.HttpException;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ExtDirectExceptionHandlerTest
{
  @Mock
  private RegisteredMethod registeredMethod;

  private ExtDirectExceptionHandler exceptionHandler;

  @Before
  public void setUp() {
    exceptionHandler = new ExtDirectExceptionHandler();
  }

  @Test
  public void handleIQConnectionException() {
    HttpException exception = new HttpException(404, "Not Found", new IllegalAccessException());

    ErrorResponse response = (ErrorResponse) exceptionHandler.handleException(registeredMethod, exception);

    assertThat(response.getMessage(), is("Connection unsuccessful."));
  }

  @Test
  public void handleNonSuppressedExceptions() {
    Exception exception = new Exception("exception message");

    ErrorResponse response = (ErrorResponse) exceptionHandler.handleException(registeredMethod, exception);

    assertThat(response.getMessage(), is(exception.getMessage()));
  }

  @Test
  public void handleConstraintValidationException() {
    Path path = mock(Path.class);
    when(path.iterator()).thenReturn(Collections.emptyIterator());

    ConstraintViolation<?> violation = mock(ConstraintViolation.class);
    when(violation.getPropertyPath()).thenReturn(path);
    when(violation.getMessage()).thenReturn("violation message");

    ConstraintViolationException exception = new ConstraintViolationException(Set.of(violation));

    ValidationResponse response = (ValidationResponse) exceptionHandler.handleException(registeredMethod, exception);

    assertThat(response.getMessages(), contains(violation.getMessage()));
  }

  @Test
  public void handleUnauthorizedException() {
    UnauthorizedException exception = new UnauthorizedException("Subject does not have permission [astrum:search:read]");

    ErrorResponse response = (ErrorResponse) exceptionHandler.handleException(registeredMethod, exception);

    assertThat(response.getMessage(), is(exception.getMessage()));
  }

  @Test
  public void handleWrappedAuthorizationException() {
    UnauthorizedException authException = new UnauthorizedException("Permission denied");
    Exception wrapper = new Exception("Wrapper", authException);

    ErrorResponse response = (ErrorResponse) exceptionHandler.handleException(registeredMethod, wrapper);

    // Verify that wrapped authorization exception returns the wrapper message
    assertThat(response.getMessage(), is("Wrapper"));
  }
}
