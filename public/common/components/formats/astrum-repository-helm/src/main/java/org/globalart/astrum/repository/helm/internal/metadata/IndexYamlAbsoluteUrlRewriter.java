
package org.globalart.astrum.repository.helm.internal.metadata;

import org.springframework.stereotype.Component;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.thread.io.StreamCopier;

import com.google.common.io.ByteStreams;

/**
 * Removes absolute URL entries from index.yaml
 *
 * @since 3.28
 */
@Component
public class IndexYamlAbsoluteUrlRewriter
    extends IndexYamlAbsoluteUrlRewriterSupport
{
  private static final String YAML_CONTENT_TYPE = "text/x-yaml";

  @Nullable
  public Content removeUrlsFromIndexYaml(final TempBlob index, final AttributesMap attributes) {
    if (index == null) {
      return null;
    }

    try (InputStream inputStream = index.get()) {
      return new StreamCopier<>(outputStream -> updateUrls(inputStream, outputStream), input -> createContent(input, attributes)).read();
    } catch (IOException ex) {
      throw new UncheckedIOException(ex);
    }
  }

  @Nullable
  public Content removeUrlsFromIndexYaml(final Content index) {
    if (index == null) {
      return null;
    }

    try (InputStream inputStream = index.openInputStream()) {
      return new StreamCopier<>(outputStream -> updateUrls(inputStream, outputStream), input -> createContent(input, index.getAttributes())).read();
    } catch (IOException ex) {
      throw new UncheckedIOException(ex);
    }
  }

  private Content createContent(final InputStream input, final AttributesMap attributes) {
    try {
      Content content = new Content(new BytesPayload(ByteStreams.toByteArray(input), YAML_CONTENT_TYPE));
      attributes.forEach(attr-> content.getAttributes().set(attr.getKey(), attr.getValue()));
      return content;
    } catch (IOException ex) {
      log.error("Error rewriting urls in index.yaml", ex);
      throw new UncheckedIOException(ex);
    }
  }
}
