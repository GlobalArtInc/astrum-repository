
package org.globalart.astrum.repository.view.handlers;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.HighAvailabilitySupportChecker;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * UT for {@link FormatHighAvailabilitySupportHandler}
 *
 * @since 3.17
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class FormatHighAvailabilitySupportHandlerTest
{
  private final String FORMAT_NAME = "dummyFormat";

  @Mock
  private Context context;

  @Mock
  private Repository repository;

  @Mock
  private Format format;

  @Mock
  private HighAvailabilitySupportChecker highAvailabilitySupportChecker;

  private FormatHighAvailabilitySupportHandler handlerUnderTest;

  @Before
  public void setup() throws Exception {
    handlerUnderTest = new FormatHighAvailabilitySupportHandler(highAvailabilitySupportChecker);
    when(context.getRepository()).thenReturn(repository);
    when(context.getRepository().getFormat()).thenReturn(format);
    when(format.getValue()).thenReturn(FORMAT_NAME);
    when(context.proceed()).thenReturn(HttpResponses.ok());
  }

  @Test
  public void responseIsOk_IfAstrumIsNotClusteredAndFormatIsNotClustered() throws Exception {
    when(highAvailabilitySupportChecker.isSupported(FORMAT_NAME)).thenReturn(true);
    final Response response = handlerUnderTest.handle(context);
    verify(highAvailabilitySupportChecker).isSupported(FORMAT_NAME);
    assertThat(response.getStatus().getCode(), is(HttpStatus.OK));
  }

  @Test
  public void responseIsBadRequest_IfAstrumIsClusteredAndFormatIsNotClustered() throws Exception {
    when(highAvailabilitySupportChecker.isSupported(FORMAT_NAME)).thenReturn(false);
    final Response response = handlerUnderTest.handle(context);
    verify(highAvailabilitySupportChecker).isSupported(FORMAT_NAME);
    assertThat(response.getStatus().getCode(), is(HttpStatus.BAD_REQUEST));
  }
}
