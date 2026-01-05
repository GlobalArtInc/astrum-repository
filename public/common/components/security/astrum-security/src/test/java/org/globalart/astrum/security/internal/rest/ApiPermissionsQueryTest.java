
package org.globalart.astrum.security.internal.rest;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;

class ApiPermissionsQueryTest
{
  private final List<ApiEndpointPermission> sample = List.of(
      new ApiEndpointPermission(
          "GET",
          "/service/rest/v1/repositories",
          List.of(new ApiPermissionRequirement("astrum:repository-admin:*:*:read", "AND")),
          "List",
          "Repository Management",
          true),
      new ApiEndpointPermission(
          "DELETE",
          "/service/rest/v1/repositories/x",
          List.of(new ApiPermissionRequirement("astrum:repositories:delete", "AND")),
          "Delete",
          "Repository Management",
          true));

  @Test
  void filtersByMethodAndPathAndPermissionAndCombined() {
    assertThat(ApiPermissionsQuery.apply(sample, "GET", null, null, null), hasSize(1));
    assertThat(ApiPermissionsQuery.apply(sample, null, "repositories", null, null), hasSize(2));
    assertThat(ApiPermissionsQuery.apply(sample, null, null, "delete", null), hasSize(1));
    assertThat(
        ApiPermissionsQuery.apply(sample, "DELETE", "repositories", "astrum:repositories", null),
        hasSize(1));
  }

  @Test
  void filtersByTagExactCaseInsensitive() {
    assertThat(ApiPermissionsQuery.apply(sample, null, null, null, "repository management"), hasSize(2));
    assertThat(ApiPermissionsQuery.apply(sample, null, null, null, "Other"), hasSize(0));
  }
}
