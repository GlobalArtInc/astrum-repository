

package org.globalart.astrum.content.maven.internal.recipe;

import java.net.URI;
import java.net.URISyntaxException;

import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.repository.maven.MavenProxyRequestHeaderSupport;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.apache.http.client.methods.HttpRequestBase;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenProxyFacetTest

{
  @Mock
  private ConstraintViolationFactory constraintViolationFactory;

  @Mock
  private ApplicationVersion applicationVersion;

  @Mock
  private MavenProxyRequestHeaderSupport mavenProxyRequestHeaderSupport;

  private MavenProxyFacet underTest;

  @Before
  public void setUp() {
    this.underTest = new MavenProxyFacet(constraintViolationFactory, mavenProxyRequestHeaderSupport);
    when(applicationVersion.getEdition()).thenReturn("edition");
  }

  @Test
  public void testNonMavenCentralHostAndVerifyRequestHeader() throws URISyntaxException {
    URI uri = new URI("schema", "host", "/path/test", "fragment");
    Context context = mock(Context.class);

    HttpRequestBase request = underTest.buildFetchHttpRequest(uri, context);
    assertEquals(request.getAllHeaders().length, 0);
  }

  @Test
  public void testMavenCentralHostAndVerifyRequestHeader() throws URISyntaxException {
    URI uri = new URI("schema", "repo1.maven.org", "/path/test", "fragment");
    Context context = mock(Context.class);
    HttpRequestBase request = underTest.buildFetchHttpRequest(uri, context);
    assertEquals(request.getAllHeaders().length, 1);
  }

  // NOTE: isNotModified() tests have been moved to ContentProxyFacetSupportTest
  // since the ETag-based blob reuse logic is now in the base ContentProxyFacetSupport class.
}
