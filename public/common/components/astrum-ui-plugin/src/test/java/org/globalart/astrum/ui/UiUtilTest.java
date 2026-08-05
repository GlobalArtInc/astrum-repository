
package org.globalart.astrum.ui;

import java.net.URL;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.springframework.context.ApplicationContext;
import org.springframework.core.io.Resource;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class UiUtilTest
{
  private static final String TEST_URL = "https://someurl/astrum-frontend-bundle.js";

  @Mock
  private Resource resource;

  @Mock
  private ApplicationContext context;

  @InjectMocks
  private UiUtil undertest;

  @Test
  public void getHashedFilename() throws Exception {
    when(resource.getURL()).thenReturn(new URL(TEST_URL));
    when(context.getResources("classpath*:/static/**/astrum-frontend-bundle.js")).thenReturn(new Resource[]{resource});

    String hashedFilename = undertest.getPathForFile("astrum-frontend-bundle.js");

    assertThat(hashedFilename, is("/astrum-frontend-bundle.js"));
  }
}
