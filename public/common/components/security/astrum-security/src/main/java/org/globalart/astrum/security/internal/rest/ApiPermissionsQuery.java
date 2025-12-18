
package org.globalart.astrum.security.internal.rest;

import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nullable;

import org.apache.commons.lang3.StringUtils;

/**
 * AND-combined filters for {@link ApiPermissionsResource} query parameters.
 */
public final class ApiPermissionsQuery
{
  private ApiPermissionsQuery() {
  }

  public static List<ApiEndpointPermission> apply(
      final List<ApiEndpointPermission> source,
      @Nullable final String method,
      @Nullable final String pathSubstring,
      @Nullable final String permissionSubstring,
      @Nullable final String tagExact)
  {
    List<ApiEndpointPermission> out = new ArrayList<>();
    for (ApiEndpointPermission row : source) {
      if (StringUtils.isNotBlank(method)
          && !row.getHttpMethod().equalsIgnoreCase(method.trim())) {
        continue;
      }
      if (StringUtils.isNotBlank(pathSubstring)
          && !StringUtils.containsIgnoreCase(row.getPathPattern(), pathSubstring.trim())) {
        continue;
      }
      if (StringUtils.isNotBlank(permissionSubstring) && !matchesPermission(row, permissionSubstring.trim())) {
        continue;
      }
      if (StringUtils.isNotBlank(tagExact)) {
        if (row.getTag() == null
            || !row.getTag().equalsIgnoreCase(tagExact.trim())) {
          continue;
        }
      }
      out.add(row);
    }
    return out;
  }

  private static boolean matchesPermission(
      final ApiEndpointPermission row,
      final String permissionNeedle)
  {
    if (row.getPermissions() == null) {
      return false;
    }
    for (ApiPermissionRequirement req : row.getPermissions()) {
      if (req.getPermission() != null
          && StringUtils.containsIgnoreCase(req.getPermission(), permissionNeedle)) {
        return true;
      }
    }
    return false;
  }
}
