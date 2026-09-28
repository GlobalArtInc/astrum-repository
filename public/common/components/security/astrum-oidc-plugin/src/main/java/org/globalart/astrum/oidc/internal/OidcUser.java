package org.globalart.astrum.oidc.internal;

import java.util.List;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public record OidcUser(
    String userId,
    String subject,
    String firstName,
    String lastName,
    String email,
    List<String> groups,
    long lastLogin)
{
  public OidcUser {
    groups = groups == null ? List.of() : List.copyOf(groups);
  }
}
