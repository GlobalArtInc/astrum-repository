package org.globalart.astrum.rapture.internal.i18n;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.util.List;

import org.globalart.astrum.webresources.WebResource;
import org.globalart.astrum.webresources.WebResourceBundle;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Serves {@code /static/rapture/i18n-<language>.js} for every language besides English.
 */
@Component
public class I18nWebResourceBundle
    implements WebResourceBundle
{
  public static final String PATH = "/static/rapture/i18n-%s.js";

  private final List<WebResource> resources;

  @Autowired
  public I18nWebResourceBundle(final UiLanguages languages) {
    checkNotNull(languages);
    this.resources = languages.available().stream()
        .filter(language -> !UiLanguages.DEFAULT.equals(language))
        .map(language -> (WebResource) new Script(String.format(PATH, language), languages.script(language)))
        .toList();
  }

  @Override
  public List<WebResource> getResources() {
    return resources;
  }

  private record Script(String path, byte[] content)
      implements WebResource
  {
    @Override
    public String getPath() {
      return path;
    }

    @Override
    public String getContentType() {
      return JAVASCRIPT;
    }

    @Override
    public long getSize() {
      return content.length;
    }

    @Override
    public long getLastModified() {
      return UNKNOWN_LAST_MODIFIED;
    }

    @Override
    public boolean isCacheable() {
      return true;
    }

    @Override
    public InputStream getInputStream() {
      return new ByteArrayInputStream(content);
    }
  }
}
