package org.globalart.astrum.rapture.internal.i18n;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Locale;

import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class UiLanguagesTest
{
  private final UiLanguages underTest = new UiLanguages();

  @Test
  public void englishIsBuiltInAndOtherLanguagesComeFromDictionaries() {
    assertThat(underTest.available().get(0), is("en"));
    assertThat(underTest.available(), hasItem("ru"));
    assertThat(underTest.isAvailable("xx"), is(false));
    assertThat(underTest.dictionaryNames("ru"), hasItem("test-dictionary"));
  }

  @Test
  public void theCookieWinsOverTheBrowser() {
    assertThat(underTest.resolve(request(new Cookie[]{new Cookie(UiLanguages.COOKIE, "en")}, "ru")), is("en"));
    assertThat(underTest.resolve(request(new Cookie[]{new Cookie(UiLanguages.COOKIE, "ru")}, "en")), is("ru"));
  }

  @Test
  public void anUnknownCookieFallsBackToTheBrowser() {
    assertThat(underTest.resolve(request(new Cookie[]{new Cookie(UiLanguages.COOKIE, "xx")}, "ru")), is("ru"));
  }

  @Test
  public void theFirstAvailableBrowserLanguageIsUsed() {
    assertThat(underTest.resolve(request(null, "de", "ru-RU", "en")), is("ru"));
    assertThat(underTest.resolve(request(null, "de", "fr")), is("en"));
    assertThat(underTest.resolve(null), is("en"));
  }

  @Test
  public void theScriptCarriesTheDictionaries() {
    String script = new String(underTest.script("ru"), StandardCharsets.UTF_8);
    assertThat(script, containsString("window.AS_I18N = "));
    assertThat(script, containsString("\"language\":\"ru\""));
    assertThat(script, containsString("\"test-dictionary\":{\"Greeting\":\"Привет\"}"));
    assertThat(script, not(containsString("Hello")));
  }

  @Test
  public void nativeNames() {
    assertThat(UiLanguages.nativeName("ru"), is("Русский"));
    assertThat(UiLanguages.nativeName("en"), is("English"));
  }

  private static HttpServletRequest request(final Cookie[] cookies, final String... languages) {
    HttpServletRequest request = mock(HttpServletRequest.class);
    when(request.getCookies()).thenReturn(cookies);
    List<Locale> locales = Arrays.stream(languages).map(Locale::forLanguageTag).toList();
    when(request.getLocales()).thenReturn(Collections.enumeration(locales));
    return request;
  }
}
