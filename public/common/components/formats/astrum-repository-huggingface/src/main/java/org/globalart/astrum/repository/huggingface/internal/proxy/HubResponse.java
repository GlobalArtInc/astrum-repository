package org.globalart.astrum.repository.huggingface.internal.proxy;

import javax.annotation.Nullable;

import org.apache.http.Header;
import org.apache.http.HttpHeaders;
import org.apache.http.HttpResponse;

import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_ERROR_CODE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_ERROR_MESSAGE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_LINKED_ETAG;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_LINKED_SIZE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_REPO_COMMIT;

/**
 * What the Hub itself answered for a request, before any redirect to its storage.
 *
 * @param linked whether the ETag came from {@code X-Linked-Etag}, the SHA-256 of an LFS file
 */
public record HubResponse(int status, @Nullable String commit, @Nullable String etag, boolean linked, long size,
                          @Nullable String errorCode, @Nullable String errorMessage)
{
  public static HubResponse of(final HttpResponse response) {
    String linkedEtag = header(response, X_LINKED_ETAG);
    String etag = normalizeEtag(linkedEtag != null ? linkedEtag : header(response, HttpHeaders.ETAG));
    String linkedSize = header(response, X_LINKED_SIZE);
    String size = linkedSize != null ? linkedSize : header(response, HttpHeaders.CONTENT_LENGTH);
    return new HubResponse(response.getStatusLine().getStatusCode(), header(response, X_REPO_COMMIT), etag,
        linkedEtag != null, parseSize(size), header(response, X_ERROR_CODE), header(response, X_ERROR_MESSAGE));
  }

  public boolean isRedirect() {
    return status == 301 || status == 302 || status == 303 || status == 307 || status == 308;
  }

  /**
   * The same answer, as found: a redirect to the storage of the Hub still describes the file.
   */
  public HubResponse found() {
    return new HubResponse(200, commit, etag, linked, size, errorCode, errorMessage);
  }

  @Nullable
  static String normalizeEtag(@Nullable final String value) {
    if (value == null) {
      return null;
    }
    String etag = value.trim();
    if (etag.startsWith("W/")) {
      etag = etag.substring(2);
    }
    if (etag.length() >= 2 && etag.startsWith("\"") && etag.endsWith("\"")) {
      etag = etag.substring(1, etag.length() - 1);
    }
    return etag.isEmpty() ? null : etag;
  }

  private static long parseSize(@Nullable final String value) {
    if (value == null) {
      return -1;
    }
    try {
      return Long.parseLong(value.trim());
    }
    catch (NumberFormatException e) {
      return -1;
    }
  }

  @Nullable
  private static String header(final HttpResponse response, final String name) {
    Header header = response.getFirstHeader(name);
    return header == null ? null : header.getValue();
  }
}
