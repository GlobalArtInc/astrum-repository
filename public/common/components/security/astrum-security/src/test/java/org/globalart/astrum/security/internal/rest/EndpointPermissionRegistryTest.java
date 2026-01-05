
package org.globalart.astrum.security.internal.rest;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Pattern;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.ObjectProvider;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;

class EndpointPermissionRegistryTest
{
  private EndpointPermissionRegistry newRegistry() {
    @SuppressWarnings("unchecked")
    ObjectProvider<org.globalart.astrum.swagger.internal.SwaggerModel> swaggerProvider =
        mock(ObjectProvider.class);
    return new EndpointPermissionRegistry(new PermissionMappingService(), swaggerProvider);
  }

  /**
   * Directly seeds the private {@code state} field (a {@code RegistryState} record) so the test
   * exercises {@link EndpointPermissionRegistry#getPermissionForEndpoint} and
   * {@link EndpointPermissionRegistry#getEndpointsForPermission} without booting a Spring context.
   */
  private void seedEndpoints(
      final EndpointPermissionRegistry registry,
      final List<ApiEndpointPermission> eps) throws Exception
  {
    Map<String, Pattern> patterns = new LinkedHashMap<>();
    for (ApiEndpointPermission ep : eps) {
      String p = ep.getPathPattern();
      if (p != null && p.contains("{")) {
        patterns.put(p, Pattern.compile(EndpointPermissionScanner.buildTemplateRegex(p)));
      }
    }

    // RegistryState is a private inner record; locate and instantiate it via reflection.
    Class<?> registryStateClass = null;
    for (Class<?> inner : EndpointPermissionRegistry.class.getDeclaredClasses()) {
      if ("RegistryState".equals(inner.getSimpleName())) {
        registryStateClass = inner;
        break;
      }
    }
    if (registryStateClass == null) {
      throw new IllegalStateException("RegistryState inner class not found in EndpointPermissionRegistry");
    }
    Constructor<?> ctor = registryStateClass.getDeclaredConstructor(List.class, Map.class);
    ctor.setAccessible(true);
    Object registryState = ctor.newInstance(eps, Map.copyOf(patterns));

    Field stateField = EndpointPermissionRegistry.class.getDeclaredField("state");
    stateField.setAccessible(true);
    stateField.set(registry, registryState);
  }

  private static ApiEndpointPermission endpoint(final String method, final String path, final String permission) {
    List<ApiPermissionRequirement> reqs = permission == null
        ? List.of()
        : List.of(new ApiPermissionRequirement(permission, "AND"));
    return new ApiEndpointPermission(method, path, reqs, null, null, permission != null);
  }

  /**
   * Test A: path-template matching — a concrete URL should resolve to the permission of the
   * registered {@code {userId}} template.
   */
  @Test
  void getPermissionForEndpoint_matchesPathTemplate() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();

    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users/{userId}", "astrum:users:read"),
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/users/admin", "GET");

    assertThat(permission, is("astrum:users:read"));
  }

  @Test
  void getPermissionForEndpoint_returnsNullForUnknownPath() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users/{userId}", "astrum:users:read")));

    assertThat(registry.getPermissionForEndpoint("/service/rest/v1/does/not/exist", "GET"), is(nullValue()));
  }

  /**
   * Test B: exact permission matching — a query for {@code astrum:ssl-truststore:read} must not
   * leak endpoints guarded by {@code astrum:ssl-truststore:delete}. This verifies Fix 2
   * (exact match, not domain-prefix match).
   */
  @Test
  void getEndpointsForPermission_exactMatchNotDomainPrefix() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();

    ApiEndpointPermission sslRead =
        endpoint("GET", "/service/rest/v1/security/ssl", "astrum:ssl-truststore:read");
    ApiEndpointPermission sslDelete =
        endpoint("DELETE", "/service/rest/v1/security/ssl/{id}", "astrum:ssl-truststore:delete");

    seedEndpoints(registry, List.of(sslRead, sslDelete));

    List<ApiEndpointPermission> results =
        registry.getEndpointsForPermission("astrum:ssl-truststore:read");

    assertThat(results, hasSize(1));
    assertThat(results, containsInAnyOrder(sslRead));
  }

  @Test
  void getEndpointsForPermission_emptyWhenNoMatch() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/ssl", "astrum:ssl-truststore:read")));

    assertThat(registry.getEndpointsForPermission("astrum:does-not-exist:read"), is(empty()));
  }

  // ===== Edge-case: null / blank inputs =====

  @Test
  void getPermissionForEndpoint_nullEndpoint_returnsNull() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    assertThat(registry.getPermissionForEndpoint(null, "GET"), is(nullValue()));
  }

  @Test
  void getPermissionForEndpoint_nullMethod_returnsNull() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    assertThat(registry.getPermissionForEndpoint("/service/rest/v1/security/users", null), is(nullValue()));
  }

  @Test
  void getEndpointsForPermission_nullPermission_returnsEmpty() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/ssl", "astrum:ssl-truststore:read")));

    assertThat(registry.getEndpointsForPermission(null), is(empty()));
  }

  // ===== Edge-case: query-string stripping =====

  @Test
  void getPermissionForEndpoint_stripsQueryString() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    // Query parameters must be stripped before lookup
    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/users?foo=bar&page=1", "GET");

    assertThat(permission, is("astrum:users:read"));
  }

  @Test
  void getPermissionForEndpoint_stripsQueryStringFromTemplatedPath() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users/{userId}", "astrum:users:read")));

    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/users/admin?active=true", "GET");

    assertThat(permission, is("astrum:users:read"));
  }

  // ===== Edge-case: trailing-slash normalisation =====

  @Test
  void getPermissionForEndpoint_stripsTrailingSlash() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    // Trailing slash on caller's URL must be stripped before lookup
    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/users/", "GET");

    assertThat(permission, is("astrum:users:read"));
  }

  @Test
  void getPermissionForEndpoint_stripsTrailingSlashFromTemplatedPath() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users/{userId}", "astrum:users:read")));

    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/users/admin/", "GET");

    assertThat(permission, is("astrum:users:read"));
  }

  @Test
  void getPermissionForEndpoint_trailingSlashAndQueryStringTogether() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("DELETE", "/service/rest/v1/security/ssl/{id}", "astrum:ssl-truststore:delete")));

    String permission =
        registry.getPermissionForEndpoint("/service/rest/v1/security/ssl/cert-abc/?confirm=true", "DELETE");

    assertThat(permission, is("astrum:ssl-truststore:delete"));
  }

  // ===== Edge-case: method case-insensitivity =====

  @Test
  void getPermissionForEndpoint_methodIsCaseInsensitive() throws Exception {
    EndpointPermissionRegistry registry = newRegistry();
    seedEndpoints(registry, List.of(
        endpoint("GET", "/service/rest/v1/security/users", "astrum:users:read")));

    // Callers may pass lowercase or mixed-case HTTP methods
    assertThat(registry.getPermissionForEndpoint("/service/rest/v1/security/users", "get"),
        is("astrum:users:read"));
    assertThat(registry.getPermissionForEndpoint("/service/rest/v1/security/users", "Get"),
        is("astrum:users:read"));
  }

  // ===== Edge-case: empty registry (pre-ready state) =====

  @Test
  void getPermissionForEndpoint_returnsNullWhenRegistryEmpty() {
    EndpointPermissionRegistry registry = newRegistry();
    // No seedEndpoints — registry is in pre-ready state (onContextRefreshed not yet fired).
    // Every endpoint must appear as null (not silently public) so callers can distinguish
    // "genuinely public endpoint" from "registry not yet populated".
    assertThat(registry.getPermissionForEndpoint("/service/rest/v1/security/users", "GET"), is(nullValue()));
  }
}
