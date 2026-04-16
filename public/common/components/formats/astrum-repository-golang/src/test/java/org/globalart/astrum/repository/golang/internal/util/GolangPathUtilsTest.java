
package org.globalart.astrum.repository.golang.internal.util;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.golang.internal.metadata.GolangAttributes;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class GolangPathUtilsTest
{
  @Mock
  private TokenMatcher.State state;

  @Before
  public void setUp() {
    when(state.getTokens()).thenReturn(ImmutableMap.of(
        "module", "github.com/!burnt!sushi/example",
        "version", "v1.0.2-!r!c1",
        "extension", "zip"
    ));
  }

  @Test
  public void tokens() {
    assertThat(GolangPathUtils.module(state), is(equalTo("github.com/!burnt!sushi/example")));
    assertThat(GolangPathUtils.version(state), is(equalTo("v1.0.2-!r!c1")));
    assertThat(GolangPathUtils.extension(state), is(equalTo("zip")));
  }

  @Test
  public void paths() {
    assertThat(GolangPathUtils.assetPath(state), is(equalTo("/github.com/!burnt!sushi/example/@v/v1.0.2-!r!c1.zip")));
    assertThat(GolangPathUtils.listPath(state), is(equalTo("/github.com/!burnt!sushi/example/@v/list")));
    assertThat(GolangPathUtils.latestPath(state), is(equalTo("/github.com/!burnt!sushi/example/@latest")));
  }

  @Test
  public void attributesAreUnescaped() {
    GolangAttributes attributes = GolangPathUtils.getAttributesFromMatcherState(state);

    assertThat(attributes.getModule(), is(equalTo("github.com/BurntSushi/example")));
    assertThat(attributes.getVersion(), is(equalTo("v1.0.2-RC1")));
  }

  @Test
  public void escapeRoundTrip() {
    assertThat(GolangPathUtils.escape("github.com/BurntSushi/toml"), is(equalTo("github.com/!burnt!sushi/toml")));
    assertThat(GolangPathUtils.unescape("github.com/!burnt!sushi/toml"), is(equalTo("github.com/BurntSushi/toml")));
  }

  @Test
  public void invalidEscapes() {
    assertThat(GolangPathUtils.unescape("github.com/BurntSushi/toml"), is(nullValue()));
    assertThat(GolangPathUtils.unescape("github.com/!!x"), is(nullValue()));
    assertThat(GolangPathUtils.unescape("github.com/x!"), is(nullValue()));
  }

  @Test
  public void validVersions() {
    GolangPathUtils.validate(attributes("example.com/m", "v1.2.3"));
    GolangPathUtils.validate(attributes("example.com/m", "v0.0.0-20240101120000-abcdef123456"));
    GolangPathUtils.validate(attributes("example.com/m", "v2.0.0+incompatible"));
  }

  @Test(expected = InvalidContentException.class)
  public void invalidVersion() {
    GolangPathUtils.validate(attributes("example.com/m", "1.2.3"));
  }

  @Test(expected = InvalidContentException.class)
  public void invalidModule() {
    GolangPathUtils.validate(attributes("example.com//m", "v1.2.3"));
  }

  private static GolangAttributes attributes(final String module, final String version) {
    GolangAttributes attributes = new GolangAttributes();
    attributes.setModule(module);
    attributes.setVersion(version);
    return attributes;
  }
}
