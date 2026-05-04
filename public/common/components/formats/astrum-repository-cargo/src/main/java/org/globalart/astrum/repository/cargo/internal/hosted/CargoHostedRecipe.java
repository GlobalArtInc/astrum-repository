/*
 * Copyright 2019, Imperva, Inc. All rights reserved.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License Version 1.0, which accompanies this
 * distribution and is available at http://www.eclipse.org/legal/epl-v10.html.
 *
 * Imperva, the Imperva logo, SecureSphere, Incapsula, CounterBreach,
 * ThreatRadar, Camouflage, Attack Analytics, Prevoty and design are trademarks
 * of Imperva, Inc. and its subsidiaries. All other brand or product names are
 * trademarks or registered trademarks of their respective owners.
 */
package org.globalart.astrum.repository.cargo.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoRecipeSupport;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(CargoHostedRecipe.NAME)
public class CargoHostedRecipe
    extends CargoRecipeSupport
{
  public static final String NAME = "cargo-hosted";

  private final CargoHostedHandlers hostedHandlers;

  @Autowired
  public CargoHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(CargoFormat.NAME) final Format format,
      final CargoHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addCommonRoutes(builder);
    builder.route(apiRoute(CargoPaths.publish(), hostedHandlers.publish));
    builder.route(apiRoute(CargoPaths.yank(), hostedHandlers.yank));
    builder.route(apiRoute(CargoPaths.unyank(), hostedHandlers.unyank));
    builder.route(apiRoute(CargoPaths.owners(), hostedHandlers.owners));
    builder.route(apiRoute(CargoPaths.search(), hostedHandlers.search));
    builder.route(contentRoute(CargoPaths.download(), lastDownloadedHandler, hostedHandlers.download));
    builder.route(contentRoute(CargoPaths.index(), hostedHandlers.index));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
