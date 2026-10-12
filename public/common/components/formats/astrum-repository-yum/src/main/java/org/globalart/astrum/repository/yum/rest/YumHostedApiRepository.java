package org.globalart.astrum.repository.yum.rest;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;
import org.globalart.astrum.repository.yum.YumFormat;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

public class YumHostedApiRepository
    extends SimpleApiHostedRepository
{
  private final YumAttributes yum;

  @JsonInclude(Include.NON_NULL)
  private final YumSigningAttributes yumSigning;

  @JsonCreator
  public YumHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("yum") final YumAttributes yum,
      @JsonProperty("yumSigning") final YumSigningAttributes yumSigning)
  {
    super(name, YumFormat.NAME, url, online, storage, cleanup, component);
    this.yum = yum;
    this.yumSigning = yumSigning;
  }

  public YumAttributes getYum() {
    return yum;
  }

  public YumSigningAttributes getYumSigning() {
    return yumSigning;
  }
}
