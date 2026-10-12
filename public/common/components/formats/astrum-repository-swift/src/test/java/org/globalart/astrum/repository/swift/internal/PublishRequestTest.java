package org.globalart.astrum.repository.swift.internal;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

class PublishRequestTest
{
  record Part(String field, byte[] bytes, Map<String, String> headers)
      implements PartPayload
  {
    Part(final String field, final String text) {
      this(field, text.getBytes(UTF_8), Map.of());
    }

    @Override
    public String getName() {
      return null;
    }

    @Override
    public String getFieldName() {
      return field;
    }

    @Override
    public boolean isFormField() {
      return false;
    }

    @Override
    public String getHeader(final String name) {
      return headers.get(name);
    }

    @Override
    public InputStream openInputStream() {
      return new ByteArrayInputStream(bytes);
    }

    @Override
    public long getSize() {
      return bytes.length;
    }

    @Nullable
    @Override
    public String getContentType() {
      return null;
    }
  }

  private final List<byte[]> ingested = new ArrayList<>();

  private final TempBlob blob = mock(TempBlob.class);

  private TempBlob ingest(final InputStream in) throws IOException {
    ingested.add(in.readAllBytes());
    return blob;
  }

  @Test
  void readsAllSections() throws IOException {
    try (PublishRequest request = PublishRequest.read(List.of(
        new Part("source-archive", "zip-bytes"),
        new Part("source-archive-signature", "sig"),
        new Part("metadata", "{\"description\":\"x\"}"),
        new Part("metadata-signature", "msig"),
        new Part("something-else", "ignored")), "cms-1.0.0", this::ingest)) {
      assertThat(new String(ingested.get(0), UTF_8), is("zip-bytes"));
      assertThat(request.getArchive(), is(blob));
      assertThat(request.getArchiveSignature(), is(Base64.getEncoder().encodeToString("sig".getBytes(UTF_8))));
      assertThat(new String(request.getMetadata(), UTF_8), is("{\"description\":\"x\"}"));
      assertThat(request.getMetadataSignature(), is(Base64.getEncoder().encodeToString("msig".getBytes(UTF_8))));
    }
    verify(blob).close();
  }

  @Test
  void decodesBase64Sections() throws IOException {
    byte[] encoded = Base64.getMimeEncoder().encode("zip-bytes".getBytes(UTF_8));
    try (PublishRequest request = PublishRequest.read(List.of(
        new Part("source-archive", encoded, Map.of("Content-Transfer-Encoding", "base64"))), null, this::ingest)) {
      assertThat(new String(ingested.get(0), UTF_8), is("zip-bytes"));
      assertThat(request.getMetadata(), is(nullValue()));
      assertThat(request.getArchiveSignature(), is(nullValue()));
    }
  }

  @Test
  void refusesIncompleteRequests() {
    assertThat(assertThrows(SwiftProblem.class, () -> PublishRequest.read(null, null, this::ingest)).getStatus(),
        is(400));
    assertThat(assertThrows(SwiftProblem.class,
        () -> PublishRequest.read(List.of(new Part("metadata", "{}")), null, this::ingest)).getStatus(), is(422));
    assertThat(assertThrows(SwiftProblem.class, () -> PublishRequest.read(List.of(
        new Part("source-archive", "zip"), new Part("source-archive-signature", "sig")), null, this::ingest))
        .getStatus(), is(400));
    assertThat(assertThrows(SwiftProblem.class, () -> PublishRequest.read(List.of(
        new Part("source-archive", "zip"), new Part("metadata-signature", "sig")), "cms-1.0.0", this::ingest))
        .getStatus(), is(422));
    verify(blob, org.mockito.Mockito.times(2)).close();
  }

  @Test
  void limitsSmallSections() {
    byte[] large = new byte[PublishRequest.MAX_PART_SIZE + 1];
    SwiftProblem problem = assertThrows(SwiftProblem.class, () -> PublishRequest.read(List.of(
        new Part("source-archive", "zip"), new Part("metadata", large, Map.of())), null, this::ingest));
    assertThat(problem.getStatus(), is(413));
  }
}
