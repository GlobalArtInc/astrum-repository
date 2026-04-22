package org.globalart.astrum.repository.composer.internal;


import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.internal.group.ComposerGroupProviderJsonHandler;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.view.*;

import java.time.OffsetDateTime;

import static java.util.Arrays.asList;
import static java.util.Collections.singletonList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpStatus.INTERNAL_SERVER_ERROR;
import static org.globalart.astrum.repository.http.HttpStatus.OK;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerGroupProviderJsonHandlerTest
{

  @Mock
  private Request request;

  @Mock
  private Context context;

  @Mock
  private Repository repository;

  @Mock
  private GroupFacet groupFacet;

  @Mock
  private Repository memberRepository1;

  @Mock
  private Repository memberRepository2;

  @Mock
  private ViewFacet memberRepository1ViewFacet;

  @Mock
  private ViewFacet memberRepository2ViewFacet;

  @Mock
  private Response response1;

  @Mock
  private Response response2;

  @Mock
  private Status status1;

  @Mock
  private Status status2;

  @Mock
  private Payload payload1;

  @Mock
  private Payload payload2;

  @Mock
  private ComposerJsonProcessor composerJsonProcessor;

  private ComposerGroupProviderJsonHandler underTest;

  @Before
  public void setUp() throws Exception {
    when(context.getRepository()).thenReturn(repository);
    when(context.getRequest()).thenReturn(request);
    when(context.getAttributes()).thenReturn(new AttributesMap());

    when(request.getAction()).thenReturn(GET);
    when(request.getAttributes()).thenReturn(new AttributesMap());
    when(request.getHeaders()).thenReturn(new Headers());

    when(repository.facet(GroupFacet.class)).thenReturn(groupFacet);
    when(groupFacet.members()).thenReturn(asList(memberRepository1, memberRepository2));

    when(memberRepository1.getName()).thenReturn("member1");
    when(memberRepository1.facet(ViewFacet.class)).thenReturn(memberRepository1ViewFacet);

    when(memberRepository2.getName()).thenReturn("member2");
    when(memberRepository2.facet(ViewFacet.class)).thenReturn(memberRepository2ViewFacet);

    when(memberRepository1ViewFacet.dispatch(request, context)).thenReturn(response1);
    when(memberRepository2ViewFacet.dispatch(request, context)).thenReturn(response2);

    when(response1.getStatus()).thenReturn(status1);
    when(response1.getPayload()).thenReturn(payload1);

    when(response2.getStatus()).thenReturn(status2);
    when(response2.getPayload()).thenReturn(payload2);

    when(status1.getCode()).thenReturn(OK);
    when(status2.getCode()).thenReturn(OK);

    underTest = new ComposerGroupProviderJsonHandler(composerJsonProcessor);
  }

  @Test
  public void mergeContents() throws Exception {
    Response result = underTest.handle(context);

    assertThat(result.getStatus(), is(notNullValue()));
    assertThat(result.getStatus().getCode(), is(OK));

    verify(composerJsonProcessor)
        .mergeProviderJson(eq(repository), eq(asList(payload1, payload2)), any(OffsetDateTime.class));
  }

  @Test
  public void ignoreNonOkResponse() throws Exception {
    when(status1.getCode()).thenReturn(INTERNAL_SERVER_ERROR);

    Response result = underTest.handle(context);

    assertThat(result.getStatus(), is(notNullValue()));
    assertThat(result.getStatus().getCode(), is(OK));

    verify(composerJsonProcessor).mergeProviderJson(eq(repository), eq(singletonList(payload2)), any(OffsetDateTime.class));
  }

  @Test
  public void ignoreResponseWithoutPayload() throws Exception {
    when(response1.getPayload()).thenReturn(null);

    Response result = underTest.handle(context);

    assertThat(result.getStatus(), is(notNullValue()));
    assertThat(result.getStatus().getCode(), is(OK));

    verify(composerJsonProcessor).mergeProviderJson(eq(repository), eq(singletonList(payload2)), any(OffsetDateTime.class));
  }
}
