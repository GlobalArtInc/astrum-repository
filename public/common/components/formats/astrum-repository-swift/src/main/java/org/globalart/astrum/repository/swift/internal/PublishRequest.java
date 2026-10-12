package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Base64;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

public final class PublishRequest
    implements AutoCloseable
{
  public static final String SOURCE_ARCHIVE = "source-archive";

  public static final String SOURCE_ARCHIVE_SIGNATURE = "source-archive-signature";

  public static final String METADATA = "metadata";

  public static final String METADATA_SIGNATURE = "metadata-signature";

  static final int MAX_PART_SIZE = 1024 * 1024;

  @FunctionalInterface
  public interface Ingester
  {
    TempBlob ingest(InputStream in) throws IOException;
  }

  private TempBlob archive;

  private byte[] archiveSignature;

  private byte[] metadata;

  private byte[] metadataSignature;

  private PublishRequest() {
  }

  public static PublishRequest read(
      @Nullable final Iterable<PartPayload> parts,
      @Nullable final String signatureFormat,
      final Ingester ingester) throws IOException
  {
    if (parts == null) {
      throw SwiftProblem.badRequest("a package release must be published as multipart/form-data");
    }
    PublishRequest request = new PublishRequest();
    try {
      for (PartPayload part : parts) {
        String field = part.getFieldName();
        if (field == null) {
          continue;
        }
        switch (field) {
          case SOURCE_ARCHIVE -> {
            if (request.archive != null) {
              throw SwiftProblem.badRequest("more than one source-archive section");
            }
            try (InputStream in = open(part)) {
              request.archive = ingester.ingest(in);
            }
          }
          case SOURCE_ARCHIVE_SIGNATURE -> request.archiveSignature = small(part, field);
          case METADATA -> request.metadata = small(part, field);
          case METADATA_SIGNATURE -> request.metadataSignature = small(part, field);
          default -> drain(part);
        }
      }
      if (request.archive == null) {
        throw SwiftProblem.unprocessable("the request has no source-archive section");
      }
      boolean signed = isPresent(request.archiveSignature) || isPresent(request.metadataSignature);
      if (signed && (signatureFormat == null || signatureFormat.isBlank())) {
        throw SwiftProblem.badRequest("a signed release needs the X-Swift-Package-Signature-Format header");
      }
      if (isPresent(request.metadataSignature) && !isPresent(request.archiveSignature)) {
        throw SwiftProblem.unprocessable("the metadata is signed but the source archive is not");
      }
      return request;
    }
    catch (IOException | RuntimeException e) {
      request.close();
      throw e;
    }
  }

  private static InputStream open(final PartPayload part) throws IOException {
    InputStream in = part.openInputStream();
    String encoding = part.getHeader("Content-Transfer-Encoding");
    if (encoding != null && "base64".equalsIgnoreCase(encoding.strip())) {
      return Base64.getMimeDecoder().wrap(in);
    }
    return in;
  }

  private static byte[] small(final PartPayload part, final String field) throws IOException {
    try (InputStream in = open(part)) {
      byte[] bytes = in.readNBytes(MAX_PART_SIZE + 1);
      if (bytes.length > MAX_PART_SIZE) {
        throw SwiftProblem.payloadTooLarge("the " + field + " section is larger than " + MAX_PART_SIZE + " bytes");
      }
      return bytes;
    }
  }

  private static void drain(final PartPayload part) throws IOException {
    try (InputStream in = part.openInputStream()) {
      in.transferTo(OutputStream.nullOutputStream());
    }
  }

  private static boolean isPresent(@Nullable final byte[] bytes) {
    return bytes != null && bytes.length > 0;
  }

  public TempBlob getArchive() {
    return archive;
  }

  @Nullable
  public String getArchiveSignature() {
    return isPresent(archiveSignature) ? Base64.getEncoder().encodeToString(archiveSignature) : null;
  }

  @Nullable
  public byte[] getMetadata() {
    return metadata;
  }

  @Nullable
  public String getMetadataSignature() {
    return isPresent(metadataSignature) ? Base64.getEncoder().encodeToString(metadataSignature) : null;
  }

  @Override
  public void close() {
    if (archive != null) {
      archive.close();
    }
  }
}
