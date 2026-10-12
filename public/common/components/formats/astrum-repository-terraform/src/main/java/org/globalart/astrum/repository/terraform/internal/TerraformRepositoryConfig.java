package org.globalart.astrum.repository.terraform.internal;

import java.util.Locale;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.security.anonymous.AnonymousHelper;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;

/**
 * The {@code terraform} attributes of a repository configuration.
 */
public final class TerraformRepositoryConfig
{
  public static final String TERRAFORM = "terraform";

  public static final String REQUIRE_AUTHENTICATION = "requireAuthentication";

  public static final String REGISTRY_HOST = "registryHost";

  public static final String UPSTREAM_HOST = "upstreamHost";

  private TerraformRepositoryConfig() {
  }

  public static boolean requireAuthentication(final Repository repository) {
    return requireAuthentication(repository.getConfiguration());
  }

  public static boolean requireAuthentication(@Nullable final Configuration configuration) {
    if (configuration == null) {
      return false;
    }
    Object value = configuration.attributes(TERRAFORM).get(REQUIRE_AUTHENTICATION);
    return value instanceof Boolean flag ? flag : Boolean.parseBoolean(String.valueOf(value));
  }

  /**
   * The host name for which this repository answers service discovery at the root of the host, normalized to lower
   * case.
   */
  public static Optional<String> registryHost(@Nullable final Configuration configuration) {
    if (configuration == null) {
      return Optional.empty();
    }
    NestedAttributesMap terraform = configuration.attributes(TERRAFORM);
    return normalizeHost(terraform.get(REGISTRY_HOST) instanceof String host ? host : null);
  }

  /**
   * The host name a proxy repository mirrors providers of, when it is set rather than taken from the remote URL.
   */
  public static Optional<String> upstreamHost(@Nullable final Configuration configuration) {
    if (configuration == null) {
      return Optional.empty();
    }
    NestedAttributesMap terraform = configuration.attributes(TERRAFORM);
    return normalizeHost(terraform.get(UPSTREAM_HOST) instanceof String host ? host : null);
  }

  public static Optional<String> normalizeHost(@Nullable final String host) {
    if (host == null || host.isBlank()) {
      return Optional.empty();
    }
    String value = host.strip().toLowerCase(Locale.ROOT);
    if (value.endsWith(":443")) {
      value = value.substring(0, value.length() - ":443".length());
    }
    return TerraformPaths.isValidHostname(value) ? Optional.of(value) : Optional.empty();
  }

  /**
   * Whether a configured registry host names the host of a request. A configured host without a port matches the
   * host on any port, so the setting survives a reverse proxy that changes the port.
   */
  public static boolean matchesHost(final String registryHost, final String serverName, final int serverPort) {
    String name = serverName.toLowerCase(Locale.ROOT);
    int colon = registryHost.lastIndexOf(':');
    if (colon < 0) {
      return registryHost.equals(name);
    }
    return registryHost.substring(0, colon).equals(name)
        && registryHost.substring(colon + 1).equals(String.valueOf(serverPort));
  }

  public static boolean isAnonymous() {
    Subject subject = SecurityUtils.getSubject();
    return subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject);
  }
}
