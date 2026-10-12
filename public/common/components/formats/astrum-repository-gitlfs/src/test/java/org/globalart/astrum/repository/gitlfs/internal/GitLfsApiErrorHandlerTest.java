package org.globalart.astrum.repository.gitlfs.internal;

import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.util.ThreadContext;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class GitLfsApiErrorHandlerTest
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  private final Subject subject = mock(Subject.class);

  private final GitLfsApiErrorHandler underTest = new GitLfsApiErrorHandler();

  @BeforeEach
  void setUp() {
    ThreadContext.bind(subject);
  }

  @AfterEach
  void tearDown() {
    ThreadContext.unbindSubject();
  }

  @Test
  void answersApiFailuresWithTheirStatusAndMessage() throws Exception {
    Response response = handle(context -> {
      throw GitLfsException.unprocessable("Invalid object id");
    });

    assertThat(response.getStatus().getCode(), is(422));
    assertThat(response.getPayload().getContentType(), is(GitLfsJson.MEDIA_TYPE));
    assertThat(json(response).get("message").asText(), is("Invalid object id"));
  }

  @Test
  void turnsPlainFailuresIntoJson() throws Exception {
    Response response = handle(context -> HttpResponses.badRequest("Missing content"));
    assertThat(response.getStatus().getCode(), is(400));
    assertThat(json(response).get("message").asText(), is("Missing content"));

    Response readOnly = handle(context -> {
      throw new IllegalOperationException("Repository is read only: lfs");
    });
    assertThat(readOnly.getStatus().getCode(), is(403));

    Response ok = HttpResponses.ok();
    assertThat(handle(context -> ok), is(ok));
  }

  @Test
  void refusesSignedInUsersWithJsonAndChallengesAnonymousOnes() throws Exception {
    Handler denied = context -> {
      throw new AuthorizationException();
    };
    when(subject.getPrincipal()).thenReturn("reader");
    when(subject.isAuthenticated()).thenReturn(true);
    Response response = handle(denied);
    assertThat(response.getStatus().getCode(), is(403));
    assertThat(json(response).get("message").asText(), is("Not permitted in repository lfs"));

    when(subject.isAuthenticated()).thenReturn(false);
    assertThrows(AuthorizationException.class, () -> handle(denied));
  }

  private Response handle(final Handler next) throws Exception {
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn("lfs");
    Context context = mock(Context.class);
    when(context.getRepository()).thenReturn(repository);
    when(context.getRequest()).thenReturn(new Request.Builder().action("POST").path("/info/lfs/locks").build());
    when(context.proceed()).thenAnswer(invocation -> next.handle(context));
    return underTest.handle(context);
  }

  private static JsonNode json(final Response response) throws IOException {
    try (InputStream in = response.getPayload().openInputStream()) {
      return MAPPER.readTree(in);
    }
  }
}
