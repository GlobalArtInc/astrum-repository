
package org.globalart.astrum.security.internal.rest;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Service for mapping Astrum permissions to REST API endpoints.
 * This provides a bidirectional mapping between permissions and endpoints.
 *
 * ASTRUM-51956: {@link EndpointPermissionRegistry} now scans {@code @RequiresPermissions} annotations
 * at runtime and is the authoritative source. This class is retained only to supply manual mapping
 * rows that the registry merges in via {@code applyManualMappings}.
 */
@Component
public class PermissionMappingService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<ApiEndpointMapping> endpointMappings;

  private final Map<String, List<ApiEndpointMapping>> permissionToEndpointsMap;

  public PermissionMappingService() {
    this.endpointMappings = initializeEndpointMappings();
    this.permissionToEndpointsMap = buildPermissionToEndpointsMap();
  }

  /**
   * Gets the required permission for an API endpoint.
   *
   * @param endpoint the API endpoint path
   * @param method the HTTP method
   * @return the required permission, or null if not found
   */
  @Nullable
  public String getPermissionForEndpoint(final String endpoint, final String method) {
    checkNotNull(endpoint);
    checkNotNull(method);

    String normalizedEndpoint = normalizeEndpoint(endpoint);
    String normalizedMethod = method.toUpperCase();

    // Try exact match first
    Optional<ApiEndpointMapping> exactMatch = endpointMappings.stream()
        .filter(m -> m.matches(normalizedEndpoint, normalizedMethod))
        .findFirst();

    if (exactMatch.isPresent()) {
      return exactMatch.get().getPermission();
    }

    // Try pattern match
    return endpointMappings.stream()
        .filter(m -> m.matchesPattern(normalizedEndpoint, normalizedMethod))
        .findFirst()
        .map(ApiEndpointMapping::getPermission)
        .orElse(null);
  }

  /**
   * Gets related API endpoints for a permission.
   *
   * @param permission the permission string
   * @return list of related endpoint mappings
   */
  public List<ApiEndpointMapping> getEndpointsForPermission(final String permission) {
    checkNotNull(permission);

    // First check for exact match
    List<ApiEndpointMapping> exactMatches = permissionToEndpointsMap.get(permission);
    if (exactMatches != null && !exactMatches.isEmpty()) {
      return exactMatches;
    }

    // Try to find similar permissions (same domain)
    String domain = extractDomain(permission);
    if (domain != null) {
      return endpointMappings.stream()
          .filter(m -> m.getPermission() != null && m.getPermission().startsWith("astrum:" + domain))
          .collect(Collectors.toList());
    }

    return List.of();
  }

  /**
   * Gets all endpoint mappings.
   */
  public List<ApiEndpointMapping> getAllEndpointMappings() {
    return List.copyOf(endpointMappings);
  }

  private String normalizeEndpoint(final String endpoint) {
    // Remove trailing slash and normalize to lowercase
    String normalized = endpoint.trim();
    if (normalized.endsWith("/")) {
      normalized = normalized.substring(0, normalized.length() - 1);
    }
    // Remove query parameters
    int queryIndex = normalized.indexOf('?');
    if (queryIndex > 0) {
      normalized = normalized.substring(0, queryIndex);
    }
    return normalized;
  }

  @Nullable
  private String extractDomain(final String permission) {
    // Permission format: astrum:domain:action
    String[] parts = permission.split(":");
    if (parts.length >= 2) {
      return parts[1];
    }
    return null;
  }

  private Map<String, List<ApiEndpointMapping>> buildPermissionToEndpointsMap() {
    Map<String, List<ApiEndpointMapping>> map = new HashMap<>();
    for (ApiEndpointMapping mapping : endpointMappings) {
      map.computeIfAbsent(mapping.getPermission(), k -> new ArrayList<>()).add(mapping);
    }
    return map;
  }

  private List<ApiEndpointMapping> initializeEndpointMappings() {
    List<ApiEndpointMapping> mappings = new ArrayList<>();

    // Repository endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/repositories",
        "astrum:repository-admin:*:*:read", "List all repositories"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/repositories/{name}",
        "astrum:repository-admin:*:*:read", "Get repository by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/repositories/{format}/{type}",
        "astrum:repository-admin:*:*:add", "Create a repository"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/repositories/{format}/{type}/{name}",
        "astrum:repository-admin:*:*:edit", "Update a repository"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/repositories/{name}",
        "astrum:repository-admin:*:*:delete", "Delete a repository"));

    // Components endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/components",
        "astrum:repository-view:*:*:browse", "List components"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/components/{id}",
        "astrum:repository-view:*:*:browse", "Get component by ID"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/components",
        "astrum:repository-view:*:*:add", "Upload component"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/components/{id}",
        "astrum:repository-view:*:*:delete", "Delete component"));

    // Assets endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/assets",
        "astrum:repository-view:*:*:browse", "List assets"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/assets/{id}",
        "astrum:repository-view:*:*:browse", "Get asset by ID"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/assets/{id}",
        "astrum:repository-view:*:*:delete", "Delete asset"));

    // Search endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/search",
        "astrum:repository-view:*:*:browse", "Search components"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/search/assets",
        "astrum:repository-view:*:*:browse", "Search assets"));

    // Security - Users endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/users",
        "astrum:users:read", "List users"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/security/users",
        "astrum:users:create", "Create user"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/users/{userId}",
        "astrum:users:update", "Update user"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/security/users/{userId}",
        "astrum:users:delete", "Delete user"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/users/{userId}/change-password",
        "astrum:users:update", "Change user password"));

    // Security - Roles endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/roles",
        "astrum:roles:read", "List roles"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/roles/{id}",
        "astrum:roles:read", "Get role by ID"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/security/roles",
        "astrum:roles:create", "Create role"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/roles/{id}",
        "astrum:roles:update", "Update role"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/security/roles/{id}",
        "astrum:roles:delete", "Delete role"));

    // Security - Privileges endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/privileges",
        "astrum:privileges:read", "List privileges"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/privileges/{privilegeName}",
        "astrum:privileges:read", "Get privilege by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/security/privileges/{type}",
        "astrum:privileges:create", "Create privilege"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/privileges/{type}/{privilegeName}",
        "astrum:privileges:update", "Update privilege"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/security/privileges/{privilegeName}",
        "astrum:privileges:delete", "Delete privilege"));

    // Security - Anonymous access
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/anonymous",
        "astrum:settings:read", "Get anonymous access settings"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/anonymous",
        "astrum:settings:update", "Update anonymous access settings"));

    // Security - Realms
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/realms/active",
        "astrum:settings:read", "Get active security realms"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/realms/active",
        "astrum:settings:update", "Set active security realms"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/realms/available",
        "astrum:settings:read", "List available security realms"));

    // Blob stores endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/blobstores",
        "astrum:blobstores:read", "List blob stores"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/blobstores/{name}",
        "astrum:blobstores:read", "Get blob store by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/blobstores/{type}",
        "astrum:blobstores:create", "Create blob store"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/blobstores/{type}/{name}",
        "astrum:blobstores:update", "Update blob store"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/blobstores/{name}",
        "astrum:blobstores:delete", "Delete blob store"));

    // Tasks endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/tasks",
        "astrum:tasks:read", "List scheduled tasks"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/tasks/{id}",
        "astrum:tasks:read", "Get task by ID"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/tasks/{id}/run",
        "astrum:tasks:start", "Run a task"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/tasks/{id}/stop",
        "astrum:tasks:stop", "Stop a task"));

    // Status endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/status",
        null, "Get system status (no auth required)"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/status/check",
        null, "Health check (no auth required)"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/status/writable",
        null, "Health check — writable (no auth required)"));

    // Email configuration
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/email",
        "astrum:settings:read", "Get email configuration"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/email",
        "astrum:settings:update", "Update email configuration"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/email/verify",
        "astrum:settings:update", "Verify email configuration"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/email",
        "astrum:settings:update", "Delete email configuration"));

    // Licensing endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/system/license",
        "astrum:licensing:read", "Get license details"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/system/license",
        "astrum:licensing:update", "Install license"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/system/license",
        "astrum:licensing:delete", "Remove license"));

    // Support endpoints
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/support/supportzip",
        "astrum:atlas:create", "Create support ZIP"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/support/supportzippath",
        "astrum:atlas:create", "Create support ZIP at path"));

    // Read-only mode
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/read-only",
        "astrum:*", "Get read-only state"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/read-only/freeze",
        "astrum:*", "Enable read-only mode"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/read-only/release",
        "astrum:*", "Disable read-only mode"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/read-only/force-release",
        "astrum:*", "Force disable read-only mode"));

    // Content selectors
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/content-selectors",
        "astrum:selectors:read", "List content selectors"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/content-selectors/{name}",
        "astrum:selectors:read", "Get content selector by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/security/content-selectors",
        "astrum:selectors:create", "Create content selector"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/content-selectors/{name}",
        "astrum:selectors:update", "Update content selector"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/security/content-selectors/{name}",
        "astrum:selectors:delete", "Delete content selector"));

    // LDAP endpoints
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/ldap",
        "astrum:ldap:read", "List LDAP servers"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/security/ldap/{name}",
        "astrum:ldap:read", "Get LDAP server by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/security/ldap",
        "astrum:ldap:create", "Create LDAP server"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/security/ldap/{name}",
        "astrum:ldap:update", "Update LDAP server"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/security/ldap/{name}",
        "astrum:ldap:delete", "Delete LDAP server"));

    // Routing rules
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/routing-rules",
        "astrum:routing-rules:read", "List routing rules"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/routing-rules/{name}",
        "astrum:routing-rules:read", "Get routing rule by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/routing-rules",
        "astrum:routing-rules:create", "Create routing rule"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/routing-rules/{name}",
        "astrum:routing-rules:update", "Update routing rule"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/routing-rules/{name}",
        "astrum:routing-rules:delete", "Delete routing rule"));

    // Script endpoints (deprecated but still available)
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/script",
        "astrum:script:*", "List scripts"));
    mappings.add(new ApiEndpointMapping("GET", "/service/rest/v1/script/{name}",
        "astrum:script:*", "Get script by name"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/script",
        "astrum:script:*", "Create script"));
    mappings.add(new ApiEndpointMapping("PUT", "/service/rest/v1/script/{name}",
        "astrum:script:*", "Update script"));
    mappings.add(new ApiEndpointMapping("DELETE", "/service/rest/v1/script/{name}",
        "astrum:script:*", "Delete script"));
    mappings.add(new ApiEndpointMapping("POST", "/service/rest/v1/script/{name}/run",
        "astrum:script:*", "Run script"));

    return mappings;
  }

  /**
   * Represents a mapping between an API endpoint and its required permission.
   */
  public static class ApiEndpointMapping
  {
    private final String method;

    private final String endpoint;

    private final String permission;

    private final String description;

    private final Pattern endpointPattern;

    public ApiEndpointMapping(
        final String method,
        final String endpoint,
        @Nullable final String permission,
        final String description)
    {
      this.method = checkNotNull(method);
      this.endpoint = checkNotNull(endpoint);
      this.permission = permission;
      this.description = checkNotNull(description);

      String patternStr = endpoint.replaceAll("\\{[^}]+\\}", "[^/]+");
      this.endpointPattern = Pattern.compile("^" + patternStr + "$");
    }

    public String getMethod() {
      return method;
    }

    public String getEndpoint() {
      return endpoint;
    }

    @Nullable
    public String getPermission() {
      return permission;
    }

    public String getDescription() {
      return description;
    }

    public boolean matches(final String endpoint, final String method) {
      return this.method.equalsIgnoreCase(method) && this.endpoint.equals(endpoint);
    }

    public boolean matchesPattern(final String endpoint, final String method) {
      return this.method.equalsIgnoreCase(method) && endpointPattern.matcher(endpoint).matches();
    }
  }
}
