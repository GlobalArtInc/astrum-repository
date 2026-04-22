
package org.globalart.astrum.repository.composer.internal;

import java.util.Map;

import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.NAME_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.PROJECT_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VENDOR_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VERSION_TOKEN;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerPathUtilsTest
{
  @Mock
  private Context context;

  @Mock
  private AttributesMap contextAttributes;

  @Mock
  private TokenMatcher.State state;

  @Mock
  private Map<String, String> tokens;

  @Test
  public void buildZipballPathFromContextWithNameToken() {
    when(context.getAttributes()).thenReturn(contextAttributes);
    when(contextAttributes.require(TokenMatcher.State.class)).thenReturn(state);
    when(state.getTokens()).thenReturn(tokens);

    when(tokens.get(VENDOR_TOKEN)).thenReturn("testvendor");
    when(tokens.get(PROJECT_TOKEN)).thenReturn("testproject");
    when(tokens.get(VERSION_TOKEN)).thenReturn("1.2.3");
    when(tokens.get(NAME_TOKEN)).thenReturn("name");

    assertThat(ComposerPathUtils.buildZipballPath(context), is("/testvendor/testproject/1.2.3/name.zip"));
  }

  @Test
  public void buildZipballPathFromContextWithoutNameToken() {
    when(context.getAttributes()).thenReturn(contextAttributes);
    when(contextAttributes.require(TokenMatcher.State.class)).thenReturn(state);
    when(state.getTokens()).thenReturn(tokens);

    when(tokens.get(VENDOR_TOKEN)).thenReturn("testvendor");
    when(tokens.get(PROJECT_TOKEN)).thenReturn("testproject");
    when(tokens.get(VERSION_TOKEN)).thenReturn("1.2.3");

    assertThat(ComposerPathUtils.buildZipballPath(context),
        is("/testvendor/testproject/1.2.3/testvendor-testproject-1.2.3.zip"));
  }

  @Test
  public void buildZipballPathFromValues() {
    when(context.getAttributes()).thenReturn(contextAttributes);
    when(contextAttributes.require(TokenMatcher.State.class)).thenReturn(state);
    when(state.getTokens()).thenReturn(tokens);

    assertThat(ComposerPathUtils.buildZipballPath("testvendor", "testproject", "1.2.3"),
        is("/testvendor/testproject/1.2.3/testvendor-testproject-1.2.3.zip"));
  }

  @Test
  public void buildProviderPathFromTokens() {
    when(context.getAttributes()).thenReturn(contextAttributes);
    when(contextAttributes.require(TokenMatcher.State.class)).thenReturn(state);
    when(state.getTokens()).thenReturn(tokens);

    when(tokens.get(VENDOR_TOKEN)).thenReturn("testvendor");
    when(tokens.get(PROJECT_TOKEN)).thenReturn("testproject");

    assertThat(ComposerPathUtils.buildProviderPath(context), is("/p/testvendor/testproject.json"));
  }

  @Test
  public void buildProviderPathFromValues() {
    assertThat(ComposerPathUtils.buildProviderPath("testvendor", "testproject"), is("/p/testvendor/testproject.json"));
  }
}
