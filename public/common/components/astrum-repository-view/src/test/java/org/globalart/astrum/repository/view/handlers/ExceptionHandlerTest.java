
package org.globalart.astrum.repository.view.handlers;

import org.globalart.astrum.repository.ConcurrentOperationException;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ExceptionHandlerTest
{
  @Mock
  private Context context;

  @Mock
  private Request request;

  private ExceptionHandler underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new ExceptionHandler();
    when(context.getRepository()).thenReturn(null);
    when(context.getRequest()).thenReturn(request);
    when(request.getAction()).thenReturn("PUT");
    when(request.getPath()).thenReturn("/some/path");
  }

  @Test
  public void concurrentOperationException_returns409() throws Exception {
    when(context.proceed()).thenThrow(new ConcurrentOperationException("Component was concurrently deleted"));

    Response response = underTest.handle(context);

    assertThat(response.getStatus().getCode(), is(HttpStatus.CONFLICT));
  }

  @Test
  public void concurrentOperationException_includesMessageInResponse() throws Exception {
    when(context.proceed()).thenThrow(new ConcurrentOperationException("Component was concurrently deleted"));

    Response response = underTest.handle(context);

    assertThat(response.getStatus().getMessage(), is("Component was concurrently deleted"));
  }

  @Test
  public void successfulProceed_propagatesResponse() throws Exception {
    when(context.proceed()).thenReturn(
        new Response.Builder().status(org.globalart.astrum.repository.view.Status.success(HttpStatus.OK)).build());

    Response response = underTest.handle(context);

    assertThat(response.getStatus().getCode(), is(HttpStatus.OK));
  }
}
