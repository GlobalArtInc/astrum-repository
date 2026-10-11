
package org.globalart.astrum.repository.content.handlers;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.capability.GlobalRepositorySettings;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCounter;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpConditions;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Headers;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import com.google.common.net.HttpHeaders;
import org.joda.time.DateTime;
import org.joda.time.DateTimeZone;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.time.Duration.ofHours;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
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

  @Mock
  private AssetDownloadCounter assetDownloadCounter;

  private AttributesMap attributes;

  private AttributesMap requestAttributes;

  private Headers requestHeaders;

  private Headers responseHeaders;

  private LastDownloadedHandler underTest;

  @Before
  public void setup() throws Exception {
    configureHappyPath();

    underTest = new LastDownloadedHandler(lastDownloadedAttributeHandler, globalRepositorySettings,
        assetDownloadCounter);
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

  @Test
  public void countsSuccessfulGetRequest() throws Exception {
    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void countsEveryDownloadWhileLastDownloadedIsThrottled() throws Exception {
    when(asset.lastDownloaded()).thenReturn(Optional.of(OffsetDateTime.now()));

    underTest.handle(context);
    underTest.handle(context);

    verify(asset, never()).tryMarkAsDownloaded();
    verify(assetDownloadCounter, times(2)).recordDownload("raw", asset);
  }

  @Test
  public void doesNotCountHeadRequest() throws Exception {
    when(request.getAction()).thenReturn(HEAD);

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void doesNotCountNotModified() throws Exception {
    when(response.getStatus()).thenReturn(new Status(true, 304));

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void doesNotCountFailure() throws Exception {
    when(response.getStatus()).thenReturn(new Status(false, 404));

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void doesNotCountOtherMethods() throws Exception {
    for (String method : new String[]{PUT, POST, DELETE, OPTIONS}) {
      when(request.getAction()).thenReturn(method);

      underTest.handle(context);
    }

    verifyNotCounted();
  }

  @Test
  public void doesNotCountContentWithoutAsset() throws Exception {
    attributes.remove(Asset.class);

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void doesNotCountWhenTheConditionalRequestIsAnsweredWithNotModified() throws Exception {
    attributes.set(Content.CONTENT_ETAG, "abc");
    stashConditions(HttpHeaders.IF_NONE_MATCH, "\"abc\"");

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void doesNotCountWhenNotModifiedSinceTheGivenDate() throws Exception {
    attributes.set(Content.CONTENT_LAST_MODIFIED, new DateTime(2020, 1, 1, 0, 0, DateTimeZone.UTC));
    stashConditions(HttpHeaders.IF_MODIFIED_SINCE, "Wed, 01 Jan 2025 00:00:00 GMT");

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void countsConditionalRequestAnsweredWithTheContent() throws Exception {
    attributes.set(Content.CONTENT_ETAG, "abc");
    stashConditions(HttpHeaders.IF_NONE_MATCH, "\"older\"");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void countsRangeStartingAtTheFirstByte() throws Exception {
    requestHeaders.set(HttpHeaders.RANGE, "bytes=0-99");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void doesNotCountRangeStartingFurtherIn() throws Exception {
    for (String range : new String[]{"bytes=100-", "bytes=100-199", "bytes=-10"}) {
      requestHeaders.set(HttpHeaders.RANGE, range);

      underTest.handle(context);
    }

    verifyNotCounted();
  }

  @Test
  public void countsSuffixRangeCoveringTheWholeContent() throws Exception {
    requestHeaders.set(HttpHeaders.RANGE, "bytes=-1000");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void doesNotCountUnsatisfiableRange() throws Exception {
    requestHeaders.set(HttpHeaders.RANGE, "bytes=5000-");

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void countsRangesTheWholeContentIsSentFor() throws Exception {
    for (String range : new String[]{"bytes=100-200,300-400", "items=100-", "bytes=abc-"}) {
      requestHeaders.set(HttpHeaders.RANGE, range);

      underTest.handle(context);
    }

    verify(assetDownloadCounter, times(3)).recordDownload("raw", asset);
  }

  @Test
  public void countsRangeWhenIfRangeDoesNotMatch() throws Exception {
    attributes.set(Content.CONTENT_ETAG, "abc");
    requestHeaders.set(HttpHeaders.RANGE, "bytes=100-");
    requestHeaders.set(HttpHeaders.IF_RANGE, "\"older\"");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void doesNotCountRangeWhenIfRangeMatches() throws Exception {
    attributes.set(Content.CONTENT_ETAG, "abc");
    requestHeaders.set(HttpHeaders.RANGE, "bytes=100-");
    requestHeaders.set(HttpHeaders.IF_RANGE, "\"abc\"");

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void countsRangeOfContentOfUnknownSize() throws Exception {
    when(payload.getSize()).thenReturn(-1L);
    requestHeaders.set(HttpHeaders.RANGE, "bytes=100-");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void countsPartialContentFromTheFirstByte() throws Exception {
    when(response.getStatus()).thenReturn(new Status(true, 206));
    responseHeaders.set(HttpHeaders.CONTENT_RANGE, "bytes 0-99/1000");

    underTest.handle(context);

    verify(assetDownloadCounter).recordDownload("raw", asset);
  }

  @Test
  public void doesNotCountPartialContentStartingFurtherIn() throws Exception {
    when(response.getStatus()).thenReturn(new Status(true, 206));
    responseHeaders.set(HttpHeaders.CONTENT_RANGE, "bytes 100-199/1000");

    underTest.handle(context);

    verifyNotCounted();
  }

  @Test
  public void returnsTheResponseWhenCountingFails() throws Exception {
    doThrow(new IllegalStateException("boom")).when(assetDownloadCounter).recordDownload(anyString(), any());

    Response handledResponse = underTest.handle(context);

    assertThat(handledResponse, is(equalTo(response)));
  }

  private void stashConditions(final String header, final String value) {
    requestHeaders.set(header, value);
    HttpConditions.makeUnconditional(request);
  }

  private void verifyNotCounted() {
    verify(assetDownloadCounter, never()).recordDownload(anyString(), any());
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
    when(repository.getFormat()).thenReturn(new Format("raw") { });
    when(request.getAction()).thenReturn(GET);
    requestAttributes = new AttributesMap();
    requestHeaders = new Headers();
    responseHeaders = new Headers();
    when(request.getAttributes()).thenReturn(requestAttributes);
    when(request.getHeaders()).thenReturn(requestHeaders);
    when(response.getHeaders()).thenReturn(responseHeaders);
    when(payload.getSize()).thenReturn(1000L);
    when(response.getPayload()).thenReturn(payload);
    when(response.getStatus()).thenReturn(new Status(true, 200));
    when(payload.getAttributes()).thenReturn(attributes);
    when(asset.lastDownloaded()).thenReturn(Optional.empty());
    when(globalRepositorySettings.getLastDownloadedInterval()).thenReturn(ofHours(12));
  }
}
