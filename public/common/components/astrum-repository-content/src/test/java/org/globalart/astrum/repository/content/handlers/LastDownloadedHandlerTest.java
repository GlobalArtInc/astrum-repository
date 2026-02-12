
package org.globalart.astrum.repository.content.handlers;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.capability.GlobalRepositorySettings;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.time.Duration.ofHours;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.OPTIONS;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

@RunWith(MockitoJUnitRunner.Silent.class)
public class LastDownloadedHandlerTest
{
  @Mock
  private Context context;

  @Mock
  private Response response;

  @Mock
  private Content payload;

  @Mock
  private FluentAsset asset;

  @Mock
  private Repository repository;

  @Mock
  private Request request;

  @Mock
  private LastDownloadedAttributeHandler lastDownloadedAttributeHandler;

  @Mock
  private GlobalRepositorySettings globalRepositorySettings;

  private AttributesMap attributes;

  private LastDownloadedHandler underTest;

  @Before
  public void setup() throws Exception {
    configureHappyPath();

    underTest = new LastDownloadedHandler(lastDownloadedAttributeHandler, globalRepositorySettings);
  }

  @Test
  public void shouldMarkAssetAsDownloadedWhenSuccessfulGetRequest() throws Exception {
    when(asset.tryMarkAsDownloaded()).thenReturn(true);

    Response handledResponse = underTest.handle(context);

    verify(asset).tryMarkAsDownloaded();
    verify(lastDownloadedAttributeHandler).writeLastDownloadedAttribute(asset);

    assertThat(handledResponse, is(equalTo(response)));
  }

  @Test
  public void shouldMarkAssetAsDownloadedWhenSuccessfulHeadRequest() throws Exception {
    when(request.getAction()).thenReturn(HEAD);
    when(asset.tryMarkAsDownloaded()).thenReturn(true);

    Response handledResponse = underTest.handle(context);

    verify(asset).tryMarkAsDownloaded();
    verify(lastDownloadedAttributeHandler).writeLastDownloadedAttribute(asset);

    assertThat(handledResponse, is(equalTo(response)));
  }

  @Test
  public void shouldMarkAssetAsDownloadedWhenNotModified() throws Exception {
    when(response.getStatus()).thenReturn(new Status(false, 304));
    when(asset.tryMarkAsDownloaded()).thenReturn(true);

    Response handledResponse = underTest.handle(context);

    verify(asset).tryMarkAsDownloaded();
    verify(lastDownloadedAttributeHandler).writeLastDownloadedAttribute(asset);

    assertThat(handledResponse, is(equalTo(response)));
  }

  @Test
  public void shouldNotMarkAssetAsDownloadedOnFailure() throws Exception {
    when(response.getStatus()).thenReturn(new Status(false, 500));

    testNoExceptionThrownAndVerifySaveNotCalled();
  }

  @Test
  public void doesNotWriteAttributeWhenThrottled() throws Exception {
    when(asset.tryMarkAsDownloaded()).thenReturn(false);

    Response handledResponse = underTest.handle(context);

    verify(asset).tryMarkAsDownloaded();
    verify(lastDownloadedAttributeHandler, never()).writeLastDownloadedAttribute(asset);

    assertThat(handledResponse, is(equalTo(response)));
  }

  @Test
  public void skipsDbCallWhenLastDownloadedIsRecent() throws Exception {
    when(asset.lastDownloaded()).thenReturn(Optional.of(OffsetDateTime.now()));

    Response handledResponse = underTest.handle(context);

    verify(asset, never()).tryMarkAsDownloaded();
    verify(lastDownloadedAttributeHandler, never()).writeLastDownloadedAttribute(asset);

    assertThat(handledResponse, is(equalTo(response)));
  }

  @Test
  public void shouldIgnoreNullContent() throws Exception {
    when(response.getPayload()).thenReturn(null);

    testNoExceptionThrownAndVerifySaveNotCalled();
  }

  @Test
  public void shouldIgnoreMissingAttribute() throws Exception {
    attributes.remove(Asset.class);

    testNoExceptionThrownAndVerifySaveNotCalled();
  }

  @Test
  public void shouldIgnoreStringContent() throws Exception {
    when(response.getPayload()).thenReturn(new StringPayload("message", null));

    testNoExceptionThrownAndVerifySaveNotCalled();
  }

  @Test
  public void onlyUpdateForGetAndHeadRequests() throws Exception {
    assertDownloadedTimeNotUpdatedFor(PUT, POST, DELETE, OPTIONS);
  }

  private void assertDownloadedTimeNotUpdatedFor(String... methods) throws Exception {
    for (String method : methods) {
      when(request.getAction()).thenReturn(method);

      testNoExceptionThrownAndVerifySaveNotCalled();
    }
  }

  private void testNoExceptionThrownAndVerifySaveNotCalled() throws Exception {
    Response handledResponse = underTest.handle(context);

    verify(asset, never()).tryMarkAsDownloaded();

    assertThat(handledResponse, is(equalTo(response)));
  }

  private void configureHappyPath() throws Exception {
    attributes = new AttributesMap();
    attributes.set(Asset.class, asset);

    when(context.proceed()).thenReturn(response);
    when(context.getRepository()).thenReturn(repository);
    when(context.getRequest()).thenReturn(request);
    when(request.getAction()).thenReturn(GET);
    when(response.getPayload()).thenReturn(payload);
    when(response.getStatus()).thenReturn(new Status(true, 200));
    when(payload.getAttributes()).thenReturn(attributes);
    when(asset.lastDownloaded()).thenReturn(Optional.empty());
    when(globalRepositorySettings.getLastDownloadedInterval()).thenReturn(ofHours(12));
  }
}
