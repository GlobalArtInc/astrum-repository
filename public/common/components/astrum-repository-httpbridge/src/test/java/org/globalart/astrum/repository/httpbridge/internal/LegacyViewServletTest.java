
package org.globalart.astrum.repository.httpbridge.internal;

import java.util.ArrayList;
import java.util.Enumeration;
import java.util.List;
import java.util.regex.Pattern;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.httpbridge.HttpResponseSender;
import org.globalart.astrum.repository.httpbridge.LegacyViewConfiguration;
import org.globalart.astrum.repository.httpbridge.LegacyViewContributor;
import org.globalart.astrum.repository.httpbridge.internal.describe.DescriptionHelper;
import org.globalart.astrum.repository.httpbridge.internal.describe.DescriptionRenderer;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.invocation.InvocationOnMock;
import org.mockito.stubbing.Answer;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.nullable;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.http.HttpStatus.NOT_FOUND;

class LegacyViewServletTest
    extends ViewServletTest
{
  private static final String LEGACY_ENABLED =
      "org.globalart.astrum.repository.httpbridge.internal.HttpBridgeModule.legacy";

  private static final String REPOSITORY_NAME = "test-repo";

  private static final String NOT_FOUND_MESSAGE = "Repository not found";

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private HttpResponseSenderSelector httpResponseSenderSelector;

  @Mock
  private DescriptionHelper descriptionHelper;

  @Mock
  private DescriptionRenderer descriptionRenderer;

  @Mock
  private HttpServletRequest request;

  @Mock
  private HttpServletResponse response;

  @Mock
  private Repository repository;

  @Mock
  private Configuration configuration;

  @Mock
  private Enumeration<String> names;

  @Mock
  private ViewFacet viewFacet;

  @Mock
  private HttpResponseSender sender;

  @Mock
  private LegacyViewContributor rawContributor;

  @Mock
  private LegacyViewConfiguration rawConfiguration;

  private List<LegacyViewContributor> legacyViewContributors;

  private LegacyViewServlet viewServlet;

  @BeforeEach
  void setup() throws Exception {
    System.setProperty(LEGACY_ENABLED, "true");

    legacyViewContributors = new ArrayList<>();
    legacyViewContributors.add(rawContributor);

    viewServlet = buildServlet();

    mockNames();
    mockRequest();
    mockRepository();
    mockContributor();
    mockResponseSender();
  }

  @Test
  void sendRequestWhenLegacyEnabled() throws Exception {
    viewServlet.doService(request, response);

    verify(sender).send(nullable(Request.class), nullable(Response.class), eq(response));
  }

  @Test
  void sendNotFoundWhenLegacyUrlAndRepositoryNotFound() throws Exception {
    when(repositoryManager.get(REPOSITORY_NAME)).thenReturn(null);
    when(request.getPathInfo()).thenReturn("/test-repo/content.txt");
    when(request.getRequestURI()).thenReturn("/content/sites/test-repo/content.txt");

    viewServlet.doService(request, response);
    ArgumentCaptor<Response> responseCaptor = ArgumentCaptor.forClass(Response.class);
    verify(sender).send(eq(null), responseCaptor.capture(), eq(response));

    assertThat(responseCaptor.getValue().getStatus().getCode(), is(equalTo(404)));
    assertThat(responseCaptor.getValue().getStatus().getMessage(), is(equalTo(NOT_FOUND_MESSAGE)));
  }

  @Test
  void sendNotFoundWhenLegacyUrlAndFormatDoesNotMatch() throws Exception {
    when(repository.getFormat()).thenReturn(new Format("yum")
    {
    });
    when(request.getPathInfo()).thenReturn("/test-repo/content.txt");
    when(request.getRequestURI()).thenReturn("/content/sites/test-repo/content.txt");

    viewServlet.doService(request, response);
    ArgumentCaptor<Response> responseCaptor = ArgumentCaptor.forClass(Response.class);
    verify(sender).send(eq(null), responseCaptor.capture(), eq(response));

    assertThat(responseCaptor.getValue().getStatus().getCode(), is(equalTo(NOT_FOUND)));
    assertThat(responseCaptor.getValue().getStatus().getMessage(), is(equalTo(NOT_FOUND_MESSAGE)));
  }

  private LegacyViewServlet buildServlet() {
    return new LegacyViewServlet(repositoryManager, httpResponseSenderSelector,
        descriptionHelper, descriptionRenderer, legacyViewContributors);
  }

  private void mockNames() {
    lenient().when(names.hasMoreElements()).thenAnswer(new Answer<Boolean>()
    {
      int count = 0;

      @Override
      public Boolean answer(final InvocationOnMock invocationOnMock) {
        return count++ == 0;
      }
    });
    lenient().when(names.nextElement()).thenReturn("header-name");
  }

  private void mockRequest() {
    lenient().when(request.getPathInfo()).thenReturn("/test-repo/remaining-path");
    lenient().when(request.getRequestURI()).thenReturn("/test-repo/remaining-path");
    lenient().when(request.getHeaderNames()).thenReturn(names);
    lenient().when(request.getHeaders("header-name")).thenReturn(names);
    lenient().when(request.getParameterNames()).thenReturn(names);
    lenient().when(request.getAttributeNames()).thenReturn(names);
    lenient().when(request.getMethod()).thenReturn("GET");
  }

  private void mockRepository() {
    lenient().when(repositoryManager.get(REPOSITORY_NAME)).thenReturn(repository);
    lenient().when(repository.getConfiguration()).thenReturn(configuration);
    lenient().when(repository.facet(ViewFacet.class)).thenReturn(viewFacet);
    lenient().when(repository.getFormat()).thenReturn(new Format("maven")
    {
    });
    lenient().when(configuration.isOnline()).thenReturn(true);
  }

  private void mockContributor() {
    lenient().when(rawContributor.contribute()).thenReturn(rawConfiguration);
    lenient().when(rawConfiguration.getRequestPattern()).thenReturn(Pattern.compile("/content/sites/.*"));
    lenient().when(rawConfiguration.getFormat()).thenReturn("raw");
  }

  private void mockResponseSender() {
    lenient().when(httpResponseSenderSelector.sender(nullable(Repository.class))).thenReturn(sender);
    lenient().when(httpResponseSenderSelector.defaultSender()).thenReturn(sender);
  }
}
