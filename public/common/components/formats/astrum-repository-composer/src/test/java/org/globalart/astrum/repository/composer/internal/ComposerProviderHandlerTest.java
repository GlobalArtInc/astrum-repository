
package org.globalart.astrum.repository.composer.internal;

import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.internal.proxy.ComposerProviderHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.composer.internal.proxy.ComposerProviderHandler.DO_NOT_REWRITE;
import static org.globalart.astrum.repository.http.HttpStatus.INTERNAL_SERVER_ERROR;
import static org.globalart.astrum.repository.http.HttpStatus.OK;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerProviderHandlerTest
{
  @Mock
  private ComposerJsonProcessor composerJsonProcessor;

  @Mock
  private Context context;

  @Mock
  private Request request;

  @Mock
  private Response response;

  @Mock
  private Repository repository;

  @Mock
  private AttributesMap requestAttributes;

  @Mock
  private Payload payload;

  @Mock
  private Payload rewrittenPayload;

  @Mock
  private Status status;

  private ComposerProviderHandler underTest;

  @Before
  public void setUp() throws Exception {
    when(composerJsonProcessor.rewriteProviderJson(repository, payload)).thenReturn(rewrittenPayload);

    when(context.getRequest()).thenReturn(request);
    when(context.getRepository()).thenReturn(repository);
    when(context.proceed()).thenReturn(response);

    when(request.getAttributes()).thenReturn(requestAttributes);
    when(requestAttributes.get(DO_NOT_REWRITE, String.class)).thenReturn("false");

    when(response.getStatus()).thenReturn(status);
    when(response.getPayload()).thenReturn(payload);
    when(status.getCode()).thenReturn(OK);

    underTest = new ComposerProviderHandler(composerJsonProcessor);
  }

  @Test
  public void handleWithRewriteDisabled() throws Exception {
    when(requestAttributes.get(DO_NOT_REWRITE, String.class)).thenReturn("true");

    Response output = underTest.handle(context);

    assertThat(output, is(response));
    verifyNoMoreInteractions(composerJsonProcessor);
  }

  @Test
  public void handleWithRewriteEnabled() throws Exception {
    Response output = underTest.handle(context);

    assertThat(output, is(not(response)));
    assertThat(output.getPayload(), is(rewrittenPayload));

    verify(composerJsonProcessor).rewriteProviderJson(repository, payload);
  }

  @Test
  public void handleWithRewriteEnabledNotOK() throws Exception {
    when(status.getCode()).thenReturn(INTERNAL_SERVER_ERROR);

    Response output = underTest.handle(context);

    assertThat(output, is(response));
    verifyNoMoreInteractions(composerJsonProcessor);
  }

  @Test
  public void handleWithRewriteEnabledNoPayload() throws Exception {
    when(response.getPayload()).thenReturn(null);

    Response output = underTest.handle(context);

    assertThat(output, is(response));
    verifyNoMoreInteractions(composerJsonProcessor);
  }
}
