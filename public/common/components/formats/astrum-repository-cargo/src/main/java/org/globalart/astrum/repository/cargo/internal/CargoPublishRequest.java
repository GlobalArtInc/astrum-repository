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
package org.globalart.astrum.repository.cargo.internal;

import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.repository.cargo.CrateCoordinates;

import com.fasterxml.jackson.databind.JsonNode;
import com.google.common.io.ByteStreams;

public final class CargoPublishRequest
{
  static final long MAX_METADATA_LENGTH = 10L * 1024 * 1024;

  private final JsonNode metadata;

  private final CrateCoordinates coordinates;

  private final InputStream crate;

  private final long crateLength;

  private CargoPublishRequest(
      final JsonNode metadata,
      final CrateCoordinates coordinates,
      final InputStream crate,
      final long crateLength)
  {
    this.metadata = metadata;
    this.coordinates = coordinates;
    this.crate = crate;
    this.crateLength = crateLength;
  }

  public static CargoPublishRequest read(final InputStream in) throws IOException {
    long metadataLength = readLength(in);
    if (metadataLength > MAX_METADATA_LENGTH) {
      throw CargoException.badRequest("Crate metadata is too large");
    }
    byte[] json = in.readNBytes((int) metadataLength);
    if (json.length != metadataLength) {
      throw CargoException.badRequest("Truncated crate metadata");
    }
    JsonNode metadata = CargoResponses.read(json);
    if (!metadata.isObject()) {
      throw CargoException.badRequest("Crate metadata must be a JSON object");
    }
    String name = CargoIndex.text(metadata, "name");
    String version = CargoIndex.text(metadata, "vers");
    if (!CrateCoordinates.isValidName(name)) {
      throw CargoException.badRequest("Invalid crate name: " + name);
    }
    if (!CrateCoordinates.isValidVersion(version)) {
      throw CargoException.badRequest("Invalid crate version: " + version);
    }
    long crateLength = readLength(in);
    if (crateLength == 0) {
      throw CargoException.badRequest("Empty crate file");
    }
    return new CargoPublishRequest(metadata, new CrateCoordinates(name, version), ByteStreams.limit(in, crateLength),
        crateLength);
  }

  private static long readLength(final InputStream in) throws IOException {
    byte[] bytes = in.readNBytes(4);
    if (bytes.length != 4) {
      throw CargoException.badRequest("Truncated publish request");
    }
    return (bytes[0] & 0xFFL) | (bytes[1] & 0xFFL) << 8 | (bytes[2] & 0xFFL) << 16 | (bytes[3] & 0xFFL) << 24;
  }

  public JsonNode getMetadata() {
    return metadata;
  }

  public CrateCoordinates getCoordinates() {
    return coordinates;
  }

  public InputStream getCrate() {
    return crate;
  }

  public long getCrateLength() {
    return crateLength;
  }
}
