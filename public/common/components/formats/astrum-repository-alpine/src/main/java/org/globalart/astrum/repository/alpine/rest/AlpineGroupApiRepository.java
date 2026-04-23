package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class AlpineGroupApiRepository
    extends SimpleApiGroupRepository
{
  private final AlpineSigningAttributes alpineSigning;

  @JsonCreator
  public AlpineGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("alpineSigning") final AlpineSigningAttributes alpineSigning)
  {
    super(name, AlpineFormat.NAME, url, online, storage, group);
    this.alpineSigning = alpineSigning;
  }

  public AlpineSigningAttributes getAlpineSigning() {
    return alpineSigning;
  }
}
