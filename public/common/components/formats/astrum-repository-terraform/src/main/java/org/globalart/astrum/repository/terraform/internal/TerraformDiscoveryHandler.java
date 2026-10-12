package org.globalart.astrum.repository.terraform.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.TerraformPaths.DISCOVERY;
import static org.globalart.astrum.repository.terraform.TerraformPaths.MODULES;
import static org.globalart.astrum.repository.terraform.TerraformPaths.PROVIDERS;

/**
 * Service discovery of one repository at {@code <repository>/.well-known/terraform.json}. The service URLs are
 * relative to the document, so the same answer holds when a reverse proxy serves the repository at the root of a
 * host of its own.
 */
@Component
public class TerraformDiscoveryHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    return TerraformResponses.json(TerraformResponses.discovery(
        TerraformPaths.relative(DISCOVERY, MODULES), TerraformPaths.relative(DISCOVERY, PROVIDERS)));
  }
}
