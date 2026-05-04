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

import java.io.InputStream;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import org.globalart.astrum.repository.cargo.CrateCoordinates;
import org.globalart.astrum.repository.cargo.internal.CargoContentFacet;
import org.globalart.astrum.repository.cargo.internal.CargoException;
import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoPublishRequest;
import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.cargo.internal.CargoSearch;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class CargoHostedHandlers
{
  private static final Logger log = LoggerFactory.getLogger(CargoHostedHandlers.class);

  public final Handler index = context -> content(context)
      .index(CargoPaths.indexName(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler download = context -> {
    CrateCoordinates coordinates = CargoPaths.coordinates(context);
    return content(context).getCrate(coordinates.getName(), coordinates.getVersion())
        .map(asset -> HttpResponses.ok(asset.download()))
        .orElseGet(() -> HttpResponses.notFound("crate " + coordinates + " not found"));
  };

  public final Handler publish = context -> {
    Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      throw CargoException.badRequest("Missing publish request body");
    }
    CargoContentFacet content = content(context);
    try (InputStream in = payload.openInputStream()) {
      CargoPublishRequest request = CargoPublishRequest.read(in);
      try (TempBlob crate = content.ingest(request.getCrate())) {
        if (crate.getBlob().getMetrics().getContentSize() != request.getCrateLength()) {
          throw CargoException.badRequest("Truncated crate file");
        }
        content.publish(request.getCoordinates(), request.getMetadata(), crate);
        log.info("Published crate {} to {}", request.getCoordinates(), context.getRepository().getName());
      }
    }
    return CargoResponses.json(Map.of("warnings",
        Map.of("invalid_categories", List.of(), "invalid_badges", List.of(), "other", List.of())));
  };

  public final Handler yank = context -> {
    CrateCoordinates coordinates = CargoPaths.coordinates(context);
    content(context).yank(coordinates.getName(), coordinates.getVersion(), true);
    return CargoResponses.ok();
  };

  public final Handler unyank = context -> {
    CrateCoordinates coordinates = CargoPaths.coordinates(context);
    content(context).yank(coordinates.getName(), coordinates.getVersion(), false);
    return CargoResponses.ok();
  };

  public final Handler search = context -> CargoResponses.json(
      content(context).search(CargoSearch.query(context), CargoSearch.perPage(context)));

  public final Handler owners = context -> {
    String name = CargoPaths.indexName(context);
    CargoContentFacet content = content(context);
    if (content.versions(name.toLowerCase(Locale.ROOT)).isEmpty()) {
      throw CargoException.notFound("crate `" + name + "` does not exist");
    }
    String method = context.getRequest().getAction();
    if (HttpMethods.GET.equals(method) || HttpMethods.HEAD.equals(method)) {
      String owner = content.publisher(name.toLowerCase(Locale.ROOT)).orElse("admin");
      return CargoResponses.json(Map.of("users", List.of(Map.of("id", 1, "login", owner, "name", owner))));
    }
    return CargoResponses.json(Map.of("ok", true, "msg",
        "crate owners are managed with repository privileges in Astrum Repository"));
  };

  private static CargoContentFacet content(final Context context) {
    return context.getRepository().facet(CargoContentFacet.class);
  }
}
