package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class AlpineHostedApiRepository
    extends SimpleApiHostedRepository
{
  private final AlpineSigningAttributes alpineSigning;

  @JsonCreator
  public AlpineHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("alpineSigning") final AlpineSigningAttributes alpineSigning)
  {
    super(name, AlpineFormat.NAME, url, online, storage, cleanup, component);
    this.alpineSigning = alpineSigning;
  }

  public AlpineSigningAttributes getAlpineSigning() {
    return alpineSigning;
  }
}
