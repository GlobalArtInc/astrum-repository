
package org.globalart.astrum.repository.rest.api;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Lightweight component suggestion for autocomplete/typeahead.
 * Contains only essential fields to minimize response size.
 */
public class SuggestXO
{
  private final String id;

  private final String name;

  private final String group;

  private final String version;

  private final String format;

  private final String repository;

  @JsonCreator
  public SuggestXO(
      @JsonProperty("id") final String id,
      @JsonProperty("name") final String name,
      @JsonProperty("group") final String group,
      @JsonProperty("version") final String version,
      @JsonProperty("format") final String format,
      @JsonProperty("repository") final String repository)
  {
    this.id = id;
    this.name = name;
    this.group = group;
    this.version = version;
    this.format = format;
    this.repository = repository;
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }

  public String getGroup() {
    return group;
  }

  public String getVersion() {
    return version;
  }

  public String getFormat() {
    return format;
  }

  public String getRepository() {
    return repository;
  }
}
