package org.globalart.astrum.repository.yum.rest;

import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.yum.YumFormat;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

public class YumGroupApiRepository
    extends SimpleApiGroupRepository
{
  @JsonInclude(Include.NON_NULL)
  private final YumSigningAttributes yumSigning;

  @JsonCreator
  public YumGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("yumSigning") final YumSigningAttributes yumSigning)
  {
    super(name, YumFormat.NAME, url, online, storage, group);
    this.yumSigning = yumSigning;
  }

  public YumSigningAttributes getYumSigning() {
    return yumSigning;
  }
}
