
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Map;

import com.fasterxml.jackson.annotation.JsonAnyGetter;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;

/**
 * @since 3.8
 */
@JsonPropertyOrder({"id", "repository", "format", "group", "name", "version", "assets"})
public interface ComponentXO
{
  String getId();

  void setId(String id);

  String getGroup();

  void setGroup(String group);

  String getName();

  void setName(String name);

  String getVersion();

  void setVersion(String version);

  String getRepository();

  void setRepository(String repository);

  String getFormat();

  void setFormat(String format);

  List<AssetXO> getAssets();

  void setAssets(List<AssetXO> assets);

  /**
   * Attributes to add the JSON payload
   *
   * @return
   */
  @JsonAnyGetter
  Map<String, Object> getExtraJsonAttributes();
}
