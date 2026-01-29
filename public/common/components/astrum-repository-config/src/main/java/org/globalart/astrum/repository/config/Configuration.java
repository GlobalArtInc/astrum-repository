
package org.globalart.astrum.repository.config;

import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;

import static com.google.common.collect.Lists.newArrayList;

/**
 * Repository configuration.
 *
 * @since 3.0
 */
public interface Configuration
    extends Cloneable
{
  List<String> SENSITIVE_FIELD_NAMES = newArrayList("applicationPassword", "password",
      "systemPassword", "secret", "yumSigning", "aptSigning");

  static void addSensitiveFieldName(String sensitiveFieldName) {
    SENSITIVE_FIELD_NAMES.add(sensitiveFieldName);
  }

  /**
   * @since 3.24
   */
  EntityId getRepositoryId();

  String getRepositoryName();

  void setRepositoryName(String repositoryName);

  String getRecipeName();

  void setRecipeName(String recipeName);

  /**
   * @return true, if repository should serve inbound requests
   */
  boolean isOnline();

  /**
   * @param online true, if repository should serve inbound requests
   */
  void setOnline(boolean online);

  EntityId getRoutingRuleId();

  void setRoutingRuleId(EntityId routingRuleId);

  @Nullable
  Map<String, Map<String, Object>> getAttributes();

  void setAttributes(@Nullable Map<String, Map<String, Object>> attributes);

  NestedAttributesMap attributes(String key);

  /**
   * Returns a deeply cloned copy. Note that Entity.entityMetadata is not deep-copied.
   */
  Configuration copy();
}
