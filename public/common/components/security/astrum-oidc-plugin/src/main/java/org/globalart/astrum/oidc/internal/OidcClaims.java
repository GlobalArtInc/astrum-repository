package org.globalart.astrum.oidc.internal;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Map;

import static com.google.common.base.Strings.isNullOrEmpty;

final class OidcClaims
{
  private OidcClaims() {
  }

  static Object find(final Map<String, Object> claims, final String path) {
    if (isNullOrEmpty(path)) {
      return null;
    }
    String name = path.trim();
    if (claims.containsKey(name)) {
      return claims.get(name);
    }
    Object current = claims;
    for (String part : name.split("\\.")) {
      if (!(current instanceof Map<?, ?> map)) {
        return null;
      }
      current = map.get(part);
    }
    return current;
  }

  static String string(final Map<String, Object> claims, final String path) {
    Object value = find(claims, path);
    if (value == null || value instanceof Map || value instanceof Collection) {
      return null;
    }
    String text = value.toString().trim();
    return text.isEmpty() ? null : text;
  }

  static List<String> strings(final Map<String, Object> claims, final String path) {
    Object value = find(claims, path);
    List<String> result = new ArrayList<>();
    if (value instanceof Collection<?> collection) {
      for (Object item : collection) {
        if (item != null && !(item instanceof Map) && !item.toString().isBlank()) {
          result.add(item.toString().trim());
        }
      }
    }
    else if (value != null && !(value instanceof Map) && !value.toString().isBlank()) {
      result.add(value.toString().trim());
    }
    return result;
  }

  static String userId(final OidcConfiguration configuration, final Map<String, Object> claims) throws OidcException {
    String claim = configuration.getUsernameClaim();
    if (!isNullOrEmpty(claim)) {
      String value = string(claims, claim);
      if (value == null) {
        throw new OidcException("The identity provider did not return the username claim '" + claim.trim() + "'");
      }
      return value;
    }
    String value = string(claims, OidcConfiguration.DEFAULT_USERNAME_CLAIM);
    if (value == null) {
      value = string(claims, "sub");
    }
    if (value == null) {
      throw new OidcException("The identity provider did not return a subject");
    }
    return value;
  }

  static String claimOrDefault(final Map<String, Object> claims, final String configured, final String fallback) {
    return string(claims, isNullOrEmpty(configured) ? fallback : configured);
  }

  static void checkExactMatch(final OidcConfiguration configuration, final Map<String, Object> claims)
      throws OidcException
  {
    for (Map.Entry<String, String> entry : configuration.getExactMatchClaims().entrySet()) {
      Object actual = find(claims, entry.getKey());
      boolean matches = actual instanceof Collection<?> collection
          ? collection.stream().anyMatch(item -> item != null && item.toString().equals(entry.getValue()))
          : actual != null && actual.toString().equals(entry.getValue());
      if (!matches) {
        throw new OidcException("Access denied: claim '" + entry.getKey() + "' does not have the required value");
      }
    }
  }
}
