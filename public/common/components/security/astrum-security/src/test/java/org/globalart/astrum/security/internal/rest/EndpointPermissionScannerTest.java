
package org.globalart.astrum.security.internal.rest;

import java.util.List;

import org.globalart.astrum.rest.Resource;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import org.apache.shiro.authz.annotation.Logical;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.junit.jupiter.api.Test;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;

class EndpointPermissionScannerTest
{
  // ASTRUM-46395: Swagger 1.x @Api(tags="...") -> OpenAPI 3.x @Tag(name="...")
  @Tag(name = "Widget API")
  @Path("v1/widgets")
  public static class SampleResource
      implements Resource
  {
    @GET
    @Path("{id}")
    @RequiresAuthentication
    @RequiresPermissions("astrum:widgets:read")
    @Operation(summary = "Get widget")
    public String getWidget(final String id) {
      return id;
    }
  }

  @Path("v1/or")
  public static class OrPermissionsResource
      implements Resource
  {
    @GET
    @RequiresPermissions(value = {"astrum:a:read", "astrum:b:read"}, logical = Logical.OR)
    public String either() {
      return "x";
    }
  }

  @Test
  void scanJoinsClassAndMethodPaths() {
    List<ApiEndpointPermission> rows = EndpointPermissionScanner.scan(List.of(SampleResource.class));

    assertThat(rows, hasSize(1));
    ApiEndpointPermission row = rows.get(0);
    assertThat(row.getHttpMethod(), is("GET"));
    assertThat(row.getPathPattern(), is("/service/rest/v1/widgets/{id}"));
    assertThat(row.getPermissions(), hasSize(1));
    assertThat(row.getPermissions().get(0).getPermission(), is("astrum:widgets:read"));
    assertThat(row.getPermissions().get(0).getLogical(), is("AND"));
    assertThat(row.getDescription(), is("Get widget"));
    assertThat(row.getTag(), is("Widget API"));
    assertThat(row.isAuthenticated(), is(true));
  }

  @Test
  void scanUsesOrLogical() {
    List<ApiEndpointPermission> rows = EndpointPermissionScanner.scan(List.of(OrPermissionsResource.class));

    assertThat(rows, hasSize(1));
    assertThat(rows.get(0).getPermissions().get(0).getLogical(), is("OR"));
    assertThat(rows.get(0).getPermissions().get(1).getLogical(), is("OR"));
  }
}
