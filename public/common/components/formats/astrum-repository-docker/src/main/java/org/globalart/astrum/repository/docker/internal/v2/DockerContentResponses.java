package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import com.google.common.io.ByteStreams;

public final class DockerContentResponses
{
  public static final String OCTET_STREAM = "application/octet-stream";

  private DockerContentResponses() {
  }

  public static Response manifest(final Payload payload, @Nullable final String reference) {
    String digest = null;
    String mediaType = payload.getContentType();
    if (payload instanceof Content content) {
      Asset asset = content.getAttributes().get(Asset.class);
      if (asset != null) {
        Object storedDigest = asset.attributes(DockerFormat.NAME).get(DockerContentFacet.ATTR_DIGEST);
        digest = storedDigest == null ? null : storedDigest.toString();
        Object storedType = asset.attributes(DockerFormat.NAME).get(DockerContentFacet.ATTR_MEDIA_TYPE);
        if (storedType != null) {
          mediaType = storedType.toString();
        }
      }
    }
    if (digest == null && DockerDigest.isDigest(reference)) {
      digest = reference;
    }
    String type = mediaType == null ? DockerManifest.OCI_MANIFEST : mediaType;
    Payload body = payload instanceof Content && type.equals(payload.getContentType())
        ? payload
        : new TypedPayload(payload, type);
    Response.Builder builder = DockerResponses.success(200).payload(body);
    if (digest != null) {
      builder.header(DockerResponses.CONTENT_DIGEST_HEADER, digest);
      builder.header("ETag", "\"" + digest + "\"");
    }
    return builder.build();
  }

  public static Response manifest(final FluentAsset asset, final String reference) {
    return manifest(asset.download(), reference);
  }

  public static Response blob(final Payload payload, final String digest) {
    return DockerResponses.success(200)
        .header(DockerResponses.CONTENT_DIGEST_HEADER, digest)
        .header("ETag", "\"" + digest + "\"")
        .header("Accept-Ranges", "bytes")
        .payload(payload instanceof Content ? payload : new TypedPayload(payload, OCTET_STREAM))
        .build();
  }

  public static byte[] read(final Payload payload, final long limit) throws IOException {
    try (InputStream in = payload.openInputStream()) {
      byte[] bytes = ByteStreams.toByteArray(ByteStreams.limit(in, limit + 1));
      if (bytes.length > limit) {
        throw new TooLargeException();
      }
      return bytes;
    }
  }

  public static class TooLargeException
      extends IOException
  {
    public TooLargeException() {
      super("content too large");
    }
  }

  private static class TypedPayload
      implements Payload
  {
    private final Payload delegate;

    private final String contentType;

    TypedPayload(final Payload delegate, final String contentType) {
      this.delegate = delegate;
      this.contentType = contentType;
    }

    @Override
    public InputStream openInputStream() throws IOException {
      return delegate.openInputStream();
    }

    @Override
    public long getSize() {
      return delegate.getSize();
    }

    @Override
    public String getContentType() {
      return contentType;
    }

    @Override
    public void close() throws IOException {
      delegate.close();
    }
  }
}
