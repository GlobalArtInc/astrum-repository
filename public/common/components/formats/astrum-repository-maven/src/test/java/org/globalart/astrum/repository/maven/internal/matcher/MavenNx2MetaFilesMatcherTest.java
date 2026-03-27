
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenNx2MetaFilesMatcherTest

{
  @Mock
  MavenPathParser mavenPathParser;

  @Mock
  MavenPath mavenPath;

  @Mock
  Context context;

  @Mock
  Request request;

  MavenNx2MetaFilesMatcher underTest;

  @Before
  public void setup() {
    when(mavenPathParser.parsePath(any())).thenReturn(mavenPath);
    when(context.getRequest()).thenReturn(request);
    when(context.getAttributes()).thenReturn(new AttributesMap());

    underTest = new MavenNx2MetaFilesMatcher(mavenPathParser);
  }

  @Test
  public void testMatches() {
    when(request.getPath()).thenReturn("/.meta/prefixes.txt");
    assertThat(underTest.matches(context), is(true));
    when(request.getPath()).thenReturn("/.meta/somethingelse.txt");
    assertThat(underTest.matches(context), is(true));
  }

  @Test
  public void testNonMatches() {
    when(request.getPath()).thenReturn("/real/content.txt");
    assertThat(underTest.matches(context), is(false));
  }
}
