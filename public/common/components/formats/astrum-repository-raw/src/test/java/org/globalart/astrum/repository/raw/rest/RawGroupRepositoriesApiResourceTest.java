
package org.globalart.astrum.repository.raw.rest;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Set;
import java.util.stream.Collectors;

import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import org.junit.Test;
import org.globalart.astrum.repository.rest.api.FormatAndType;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_NOT_FOUND;
import static org.globalart.astrum.rest.ApiDocConstants.SUCCESS;

public class RawGroupRepositoriesApiResourceTest
{
  private Method getRepositoryMethod;

  @Test
  public void testGetRepositoryHasApiResponsesAnnotation() throws NoSuchMethodException {
    getRepositoryMethod = RawGroupRepositoriesApiResource.class
        .getMethod("getRepository", FormatAndType.class, String.class);

    ApiResponses apiResponses = getRepositoryMethod.getAnnotation(ApiResponses.class);

    assertThat("@ApiResponses annotation must be present on getRepository method",
        apiResponses, is(notNullValue()));
  }

  @Test
  public void testGetRepositoryDocumentsAllResponseCodes() throws NoSuchMethodException {
    getRepositoryMethod = RawGroupRepositoriesApiResource.class
        .getMethod("getRepository", FormatAndType.class, String.class);

    ApiResponses apiResponses = getRepositoryMethod.getAnnotation(ApiResponses.class);
    assertThat("@ApiResponses must be present", apiResponses, is(notNullValue()));

    Set<String> responseCodes = Arrays.stream(apiResponses.value())
        .map(ApiResponse::responseCode)
        .collect(Collectors.toSet());

    assertThat("Response code 200 must be documented", responseCodes, hasItem("200"));
    assertThat("Response code 401 must be documented", responseCodes, hasItem("401"));
    assertThat("Response code 403 must be documented", responseCodes, hasItem("403"));
    assertThat("Response code 404 must be documented", responseCodes, hasItem("404"));
  }

  @Test
  public void testGetRepositoryResponsesUseCorrectConstants() throws NoSuchMethodException {
    getRepositoryMethod = RawGroupRepositoriesApiResource.class
        .getMethod("getRepository", FormatAndType.class, String.class);

    ApiResponses apiResponses = getRepositoryMethod.getAnnotation(ApiResponses.class);
    assertThat("@ApiResponses must be present", apiResponses, is(notNullValue()));

    for (ApiResponse response : apiResponses.value()) {
      String code = response.responseCode();
      String description = response.description();

      assertThat("Response description must not be null", description, is(notNullValue()));
      assertThat("Response description must not be empty", description.isEmpty(), is(false));

      // Verify descriptions match constants
      switch (code) {
        case "200":
          assertThat("200 response must use SUCCESS constant", description, is(SUCCESS));
          break;
        case "401":
          assertThat("401 response must use AUTHENTICATION_REQUIRED constant",
              description, is(AUTHENTICATION_REQUIRED));
          break;
        case "403":
          assertThat("403 response must use INSUFFICIENT_PERMISSIONS constant",
              description, is(INSUFFICIENT_PERMISSIONS));
          break;
        case "404":
          assertThat("404 response must use REPOSITORY_NOT_FOUND constant",
              description, is(REPOSITORY_NOT_FOUND));
          break;
      }
    }
  }

  @Test
  public void testGetRepositoryHasExactlyExpectedResponseCodes() throws NoSuchMethodException {
    getRepositoryMethod = RawGroupRepositoriesApiResource.class
        .getMethod("getRepository", FormatAndType.class, String.class);

    ApiResponses apiResponses = getRepositoryMethod.getAnnotation(ApiResponses.class);
    assertThat("@ApiResponses must be present", apiResponses, is(notNullValue()));

    Set<String> expectedCodes = Set.of("200", "401", "403", "404");
    Set<String> actualCodes = Arrays.stream(apiResponses.value())
        .map(ApiResponse::responseCode)
        .collect(Collectors.toSet());

    assertThat("Response codes must match expected set exactly",
        actualCodes, is(expectedCodes));
  }
}
