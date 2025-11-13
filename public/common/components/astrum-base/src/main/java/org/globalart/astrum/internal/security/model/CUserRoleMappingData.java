
package org.globalart.astrum.internal.security.model;

import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.security.config.CUserRoleMapping;

import com.fasterxml.jackson.annotation.JsonIgnore;

import static com.google.common.collect.Sets.newHashSet;
import static org.globalart.astrum.common.text.Strings2.lower;
import static org.globalart.astrum.security.config.SecuritySourceUtil.isCaseInsensitiveSource;

/**
 * {@link CUserRoleMapping} data.
 *
 * @since 3.21
 */
public class CUserRoleMappingData
    implements CUserRoleMapping
{
  private Set<String> roles;

  private String source;

  private String userId;

  private int version = 1;

  @Override
  public void addRole(final String string) {
    getRoles().add(string);
  }

  @Override
  public CUserRoleMappingData clone() {
    try {
      CUserRoleMappingData copy = (CUserRoleMappingData) super.clone();

      if (this.roles != null) {
        copy.roles = newHashSet(this.roles);
      }

      return copy;
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public Set<String> getRoles() {
    if (this.roles == null) {
      this.roles = newHashSet();
    }
    return this.roles;
  }

  @Override
  public String getSource() {
    return this.source;
  }

  @Override
  public String getUserId() {
    return this.userId;
  }

  @Override
  public String getVersion() {
    return Integer.toString(version);
  }

  @Override
  public void removeRole(final String string) {
    getRoles().remove(string);
  }

  @Override
  public void setRoles(final Set<String> roles) {
    this.roles = roles;
  }

  @Override
  public void setSource(final String source) {
    this.source = source;
  }

  @Override
  public void setUserId(final String userId) {
    this.userId = userId;
  }

  @Override
  public void setVersion(final String version) {
    this.version = version != null ? Integer.parseInt(version) : 0;
  }

  /**
   * Returns lower-case userId if we need to use case-insensitive search.
   */
  @JsonIgnore
  @Nullable
  public String getUserLo() {
    return isCaseInsensitiveSource(source) ? lower(this.userId) : null;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "userId='" + userId + '\'' +
        ", source='" + source + '\'' +
        ", roles=" + roles +
        ", version='" + version + '\'' +
        '}';
  }
}
