package org.globalart.astrum.repository.rubygems.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.rubygems.internal.index.CompactIndex;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.joda.time.DateTime;

import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

public final class RubygemsResponses
{
  public static final String TEXT_UTF8 = "text/plain; charset=utf-8";

  public static final String BINARY = "application/octet-stream";

  private RubygemsResponses() {
  }

  public static Content content(final byte[] bytes, final String contentType, @Nullable final DateTime lastModified) {
    Content content = new Content(new BytesPayload(bytes, contentType));
    content.getAttributes().set(Content.CONTENT_ETAG, CompactIndex.md5(bytes));
    if (lastModified != null) {
      content.getAttributes().set(Content.CONTENT_LAST_MODIFIED, lastModified);
    }
    return content;
  }

  public static Content withMd5Etag(final Content content) {
    Asset asset = content.getAttributes().get(Asset.class);
    if (asset != null) {
      asset.blob()
          .map(AssetBlob::checksums)
          .map(checksums -> checksums.get("md5"))
          .ifPresent(md5 -> content.getAttributes().set(Content.CONTENT_ETAG, md5));
    }
    return content;
  }

  public static Optional<String> md5(final Content content) {
    withMd5Etag(content);
    return Optional.ofNullable(content.getAttributes().get(Content.CONTENT_ETAG, String.class));
  }

  public static Response text(final int status, final String message) {
    Status responseStatus = status < 400 ? Status.success(status) : Status.failure(status, message);
    return new Response.Builder()
        .status(responseStatus)
        .payload(new StringPayload(message, TEXT_PLAIN))
        .build();
  }

  @Nullable
  public static byte[] read(final Response response) throws IOException {
    Payload payload = response.getPayload();
    if (!response.getStatus().isSuccessful() || payload == null) {
      return null;
    }
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      return in.readAllBytes();
    }
  }
}
