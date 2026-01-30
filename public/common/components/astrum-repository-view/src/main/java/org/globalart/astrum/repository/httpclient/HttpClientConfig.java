
package org.globalart.astrum.repository.httpclient;

import javax.annotation.Nullable;
import jakarta.validation.Valid;

import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;

public class HttpClientConfig
{
  @Valid
  @Nullable
  public ConnectionConfiguration connection;

  @Valid
  @Nullable
  public AuthenticationConfiguration authentication;

  @Nullable
  public Boolean blocked;

  @Nullable
  public Boolean autoBlock;
}
