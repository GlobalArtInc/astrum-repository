package org.globalart.astrum.rapture.internal.i18n;

import java.util.LinkedHashMap;
import java.util.Map;

import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.rapture.StateContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Tells the UI which language it is shown in and which ones it can switch to, as state {@value #STATE_ID}.
 */
@Component
public class UiLanguageStateContributor
    implements StateContributor
{
  static final String STATE_ID = "uiLanguage";

  private final UiLanguages languages;

  private final Provider<HttpServletRequest> request;

  @Autowired
  public UiLanguageStateContributor(final UiLanguages languages, final Provider<HttpServletRequest> request) {
    this.languages = checkNotNull(languages);
    this.request = checkNotNull(request);
  }

  @Override
  public Map<String, Object> getState() {
    Map<String, Object> state = new LinkedHashMap<>();
    state.put("current", languages.resolve(currentRequest()));
    state.put("cookie", UiLanguages.COOKIE);
    state.put("available", languages.available().stream()
        .map(language -> Map.of("id", language, "name", UiLanguages.nativeName(language)))
        .toList());
    return Map.of(STATE_ID, state);
  }

  private HttpServletRequest currentRequest() {
    try {
      return request.get();
    }
    catch (RuntimeException e) {
      return null;
    }
  }
}
