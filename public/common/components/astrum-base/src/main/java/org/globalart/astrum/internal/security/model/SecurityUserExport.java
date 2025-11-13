
package org.globalart.astrum.internal.security.model;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;

import static java.util.function.Function.identity;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link CRole} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("securityUserExport")
public class SecurityUserExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final SecurityConfiguration configuration;

  @Autowired
  public SecurityUserExport(final SecurityConfiguration configuration) {
    this.configuration = configuration;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export CUser and CUserRoleMapping data to {}", file);
    Map<String, CUser> userIdToCUser = configuration.getUsers()
        .stream()
        .collect(Collectors.toMap(CUser::getId, identity()));
    List<CUserRoleMapping> userRoleMappings = configuration.getUserRoleMappings();
    List<SecurityUserData> securityUsers = new ArrayList<>(userIdToCUser.size());
    for (Entry<String, CUser> userEntry : userIdToCUser.entrySet()) {
      List<CUserRoleMapping> roleMappings = userRoleMappings.stream()
          .filter(user -> user.getUserId().equals(userEntry.getKey()))
          .collect(Collectors.toList());
      SecurityUserData securityUserData = new SecurityUserData();
      securityUserData.setUser(userEntry.getValue());
      securityUserData.setUserRoleMappings(roleMappings);
      securityUsers.add(securityUserData);
    }

    exportToJson(securityUsers, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring CUser and CUserRoleMapping data from {}", file);
    List<SecurityUserData> securityUsers = importFromJson(file, SecurityUserData.class);
    for (SecurityUserData securityUser : securityUsers) {
      configuration.addUser(securityUser.user, securityUser.getRoles());
    }
  }

  public static class SecurityUserData
  {
    @JsonProperty
    @JsonDeserialize(as = CUserData.class)
    private CUser user;

    @JsonProperty
    @JsonDeserialize(contentAs = CUserRoleMappingData.class)
    private List<CUserRoleMapping> userRoleMappings;

    public CUser getUser() {
      return user;
    }

    public void setUser(final CUser user) {
      this.user = user;
    }

    public List<CUserRoleMapping> getUserRoleMappings() {
      return userRoleMappings;
    }

    public void setUserRoleMappings(final List<CUserRoleMapping> userRoleMappings) {
      this.userRoleMappings = userRoleMappings;
    }

    @JsonIgnore
    public Set<String> getRoles() {
      return userRoleMappings.stream()
          .filter(role -> DEFAULT_SOURCE.equals(role.getSource()))
          .findFirst()
          .map(CUserRoleMapping::getRoles)
          .orElse(Collections.emptySet());
    }
  }
}
